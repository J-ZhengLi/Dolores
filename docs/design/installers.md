# Native installer packaging

Windows x64 is the supported first-alpha release target. The installer is an
unsigned Inno Setup EXE. A stable application ID permits upgrades in the selected
scope. The install-mode dialog offers **Only for me** (no elevation, account-local
Programs directory) and **For all users** (elevation, Program Files). Start-menu
shortcuts follow that scope; the desktop shortcut is optional. Both modes have
an uninstall entry. This follows Inno's [scope override](https://jrsoftware.org/ishelp/topic_setup_privilegesrequiredoverridesallowed.htm)
and [automatic directory constants](https://jrsoftware.org/ishelp/topic_consts.htm).

The installer refuses unsupported Windows architectures and explains missing
Microsoft C++ x64 runtime files with the official download link before installing.
It does not download/install the runtime automatically. Its shortcut uses the
maintained startup helper, which also checks runtime files before launching.
File-presence checks do not establish runtime version/loadability.

Uninstall displays an unchecked **Remove my conversations, settings and
attachments** checkbox. The default and silent uninstall retain local data.
Checking it removes only the account running the uninstaller's default
`%APPDATA%/dev.dolores.desktop` directory after application removal. This is the
bridge's Windows RoamingAppData location, independent of installation scope.
The dialog displays that exact location and describes exclusions. A linked data
directory/member or a held `dolores.lock` refuses removal with recovery guidance.
No other user profile is enumerated. Custom `DOLORES_DATA_DIR` locations, external
projects, shared `.agents/skills` and OS-vault credentials are retained; remove
remembered connections in the app before uninstall if their credentials should
also be deleted. UAC using another account applies to that account's data.
The installer makes no claim to erase all traces or securely overwrite files.

Windows assembly retains the portable ZIP as an **internal verified payload**.
It reuses the existing exact runtime allowlist, x64 checks, path/privacy scan,
dependency notices and per-file hashes. The installer verifies it again before
staging and compilation. Only the EXE and its checksum are exposed as release
assets; staging/source-machine folders are excluded. Missing compiler, damaged
archive, wrong version and existing output directories refuse without replacing
earlier artifacts. Build with `desktop.py build` first; `setup-inno.py` installs
the hash-pinned build-only compiler under ignored output.

```text
python scripts/ci-windows.py check --full
python scripts/ci-windows.py package
python scripts/setup-inno.py
python scripts/installer.py windows --archive <verified-payload.zip> --directory <fresh-absolute-output-directory>
```

For experimental macOS/Linux packaging, run a normal `desktop.py build` on that
native host, collect platform/runtime-bound notices with
`collect-native-notices.py --flutter-sdk <sdk> --directory <fresh-notices-dir>`,
then run `installer.py macos` or `installer.py linux` with `--notices <notices-dir>`
and `--directory <fresh-output-dir>`. Wrong-host requests and foreign/stale notices
refuse. PKG installs `Dolores.app` under Applications; DEB installs the complete
runtime under `/opt/dolores`, adds an application-menu entry/icon and declares
GTK/Secret Service/C++ prerequisites. Internal macOS framework links are retained;
escaping links and private/database/debug artifacts refuse. Prerelease names
remain in filenames; native package metadata uses OS-compatible version formats.

`test-installer-native.py --archive <verified-payload.zip> --directory <fresh-dir>`
compiles a diagnostic installer with a unique application ID/shortcut and a fixed
synthetic data folder. It exercises actual current-user install/uninstall,
silent retention, held-lock/junction refusal and explicit fixture-only removal
recovery. The fixture purge switch is absent from production compilation. It
never launches the app or uses the normal data folder. This is lifecycle/safety
evidence, not visual acceptance of the production checkbox or UAC dialog.

These commands provide packaging support, not platform qualification. macOS
signing/notarization, DEB target-distribution dependencies, app launch, credentials,
input/accessibility and install/uninstall behavior require native tests before
adding either platform to the release workflow. Neither deletes user data on
uninstall. Windows global-mode elevation, interactive checkbox/removal behavior,
clean-machine runtime recovery and signed distribution remain separate checks.
