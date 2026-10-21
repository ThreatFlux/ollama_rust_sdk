#!/usr/bin/env python3
"""Exercise Ollama CLI hostname lookup and certificate validation in its image."""

import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import ssl
import subprocess
import tempfile
import threading


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path != "/api/tags":
            self.send_error(404)
            return
        self.server.requests.append(self.path)
        response = b'{"models":[]}'
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(response)))
        self.end_headers()
        self.wfile.write(response)

    def log_message(self, *_):
        pass


def create_certificate(root):
    certificate, key = root / "ca.crt", root / "key.pem"
    commands = [
        ["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
         "-subj", "/CN=Temporary fixture CA", "-addext", "basicConstraints=critical,CA:TRUE",
         "-keyout", str(root / "ca.key"), "-out", str(certificate)],
        ["openssl", "req", "-new", "-newkey", "rsa:2048", "-nodes",
         "-subj", "/CN=host.docker.internal", "-keyout", str(key), "-out", str(root / "leaf.csr")],
        ["openssl", "x509", "-req", "-days", "1", "-in", str(root / "leaf.csr"),
         "-CA", str(certificate), "-CAkey", str(root / "ca.key"), "-CAcreateserial",
         "-extfile", str(root / "extensions.cnf"), "-out", str(root / "leaf.crt")],
    ]
    (root / "extensions.cnf").write_text(
        "basicConstraints=critical,CA:FALSE\nsubjectAltName=DNS:host.docker.internal\n"
        "keyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n")
    for command in commands:
        subprocess.run(command, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    certificate.chmod(0o644)
    return certificate, key


def run_cli(image, platform, port, certificate=None, hostname="host.docker.internal"):
    command = ["docker", "run", "--rm", "--read-only", "--cap-drop", "ALL",
               "--security-opt", "no-new-privileges", "--add-host", f"{hostname}:host-gateway"]
    if platform:
        command += ["--platform", platform]
    if certificate:
        command += ["-v", f"{certificate}:/fixture/ca.crt:ro", "-e", "SSL_CERT_FILE=/fixture/ca.crt"]
    return subprocess.run(command + ["--", image, "--url", f"https://{hostname}:{port}", "models", "list"],
                          capture_output=True, text=True, timeout=45)


def check_tls(image, platform, server, certificate):
    trusted = run_cli(image, platform, server.server_port, certificate)
    if trusted.returncode or "Available models:" not in trusted.stdout or len(server.requests) != 1:
        raise RuntimeError(f"trusted HTTPS failed: {trusted.stdout}\n{trusted.stderr}")
    untrusted = run_cli(image, platform, server.server_port)
    if not untrusted.returncode or len(server.requests) != 1:
        raise RuntimeError(f"untrusted certificate was accepted: {untrusted.stderr}")
    if not any(cause in untrusted.stderr.lower() for cause in ("unknownissuer", "unknown issuer", "self-signed certificate")):
        raise RuntimeError(f"expected certificate-specific trust failure: {untrusted.stderr}")
    mismatch = run_cli(image, platform, server.server_port, certificate, "wrong.fixture")
    if not mismatch.returncode or len(server.requests) != 1:
        raise RuntimeError(f"wrong certificate hostname was accepted: {mismatch.stderr}")
    if not any(cause in mismatch.stderr.lower() for cause in ("notvalidforname", "not valid for name", "hostname mismatch")):
        raise RuntimeError(f"expected certificate-specific hostname failure: {mismatch.stderr}")


def check_image(image, platform, root):
    certificate, key = create_certificate(root)
    # Bridge-network containers must reach this temporary host-only test service.
    server = ThreadingHTTPServer(("0.0.0.0", 0), Handler)
    server.requests = []
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.minimum_version = ssl.TLSVersion.TLSv1_2
    context.load_cert_chain(root / "leaf.crt", key)
    server.socket = context.wrap_socket(server.socket, server_side=True)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        check_tls(image, platform, server, certificate)
        print(f"Ollama container HTTPS verified: {image}; trusted request, untrusted/hostname rejection")
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image")
    parser.add_argument("--platform", choices=("linux/amd64", "linux/arm64"))
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="ollama-https-") as directory:
        check_image(args.image, args.platform, Path(directory))


if __name__ == "__main__":
    main()
