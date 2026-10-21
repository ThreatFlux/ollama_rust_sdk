# Validation-script annotation review

Reviewed all 22 annotations from [Codacy check 119072190959](https://github.com/ThreatFlux/ollama_rust_sdk/runs/119072190959) on PR 105, head e3f6956. One Markdown blank-line finding was valid and is fixed. The remaining 21 are scoped security audit false positives, grouped below. The scripts now carry line-scoped, rule-specific Bandit and Opengrep annotations for these reviewed false positives. No scanner or repository-wide rule is disabled.

| Annotations | Scope | Evidence and disposition |
| --- | --- | --- |
| 2 | `subprocess` imports, one in each validator | Importing the standard module is not itself an exploit. These test tools intentionally invoke Docker and OpenSSL. Actual call sites are reviewed below. |
| 13 | `check_static_image.py` process-call audits | Every invocation uses an argv list, explicit `shell=False`, and a timeout. Docker and OpenSSL are resolved to absolute paths from fixed tool names before execution. OpenSSL arguments are fixed and certificate bytes arrive on stdin. Image references are explicitly separated from Docker create options using `--`. Inspection, export and cleanup consume Docker-generated container/image IDs after an explicit `--` option terminator. Archive members are read individually without filesystem extraction. No repository content becomes shell code. |
| 5 | `smoke_container_https.py` process-call audits | Certificate commands use resolved tool paths, fixed argv lists, generated temporary filenames, explicit `shell=False`, and timeouts. Docker arguments are constructed as an argv list; the caller-supplied image follows `--`, and the fixture hostname/port are controlled by this script. It uses no real API credentials. Variable argv and filenames do not create command injection. |
| 1 | HTTPS fixture binds `0.0.0.0` | Intentional ephemeral TLS test listener, required for bridge-network containers to reach the host through Docker host-gateway on Linux and Docker Desktop. It serves only a fixed synthetic `/api/tags` response; other paths are rejected. It does not expose credentials, arbitrary files, or privileged operations. The listener is closed in `finally`. |

The fixture uses a fresh temporary CA and a separate signed server certificate, including a hostname SAN and serverAuth extension. Both ARM64 and AMD64 actual CLI images successfully list models through trusted HTTPS; unknown-issuer and wrong-hostname cases fail for certificate-specific causes and never reach the HTTP handler. Both images also pass static ELF, native linker provenance, fixed CA data/package-record checks and fresh Trivy 0.76.0 OS/Cargo scans with zero findings at every severity. Full host CI and unchanged MSRV passed.

No shell invocation, scanner-wide suppression, lowered severity threshold, certificate-verification bypass, or fake package version was introduced to address these annotations.

## Executable review annotations

Bandit annotations name only `B404` (the intentional subprocess import), `B603`
(reviewed argument-list execution), or `B104` (the ephemeral bridge-accessible TLS fixture).
Opengrep annotations name only the subprocess audit rules reported by Codacy:
`python_exec_rule-subprocess-call`, `python_exec_rule-subprocess-call-array`,
`python.lang.security.audit.dangerous-subprocess-use-audit.dangerous-subprocess-use-audit`,
and `python.lang.security.audit.dangerous-subprocess-use-tainted-env-args.dangerous-subprocess-use-tainted-env-args`.
Each annotation applies to one call site, with its input provenance documented nearby.
Other security rules still apply to both scripts. Function docstrings describe the validation
contracts, including rejection of untrusted issuers and mismatched certificate hostnames.
