# Public repository privacy

Raw benchmarks, screenshots, runtime traces, local databases, audit reports and recovery backups stay in ignored local output directories. Machine reports can contain hardware fingerprints, OS locale, absolute paths and precise activity timestamps. Do not force-add these artifacts. Public performance notes retain only aggregate observations and generic test context; raw reports are private and cannot be independently inspected from this repository.

Commit author and committer identities use the repository-local neutral identity. This does not change global Git settings. Before publishing, inspect all branches/tags, commit metadata and every publishable file, including untracked files intended for addition. Keep API keys in the OS credential store or process memory and exclude environment/data files. Existing credential-like strings in provider tests are synthetic validation fixtures, not production credentials.

The privacy cleanup removed raw benchmark files from every existing commit, scrubbed identifying machine details/timestamps from documentation, and anonymized both commit identities. Local Gitleaks and personal-data checks cover reachable history and a publishable working-tree snapshot. These checks are not a guarantee that every possible secret or piece of personal data can be detected.

Local privacy recovery material is kept under ignored output/privacy. It contains the original private history and must never be uploaded or included in release bundles. Review source archives and packaged application files separately; Git checks do not cover ignored user data or private output.
