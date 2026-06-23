import os
import platform
import subprocess
import sys


def build_project():
    current_os = platform.system().lower()

    if "windows" in current_os:
        script_name = "build.bat"
        if os.path.exists(script_name):
            try:
                subprocess.run([script_name], check=True, shell=True)
            except subprocess.CalledProcessError as e:
                print(f"[!] Error executing {script_name}")
                sys.exit(e.returncode)
        else:
            print(f"[!] Error: {script_name} not found.")
            sys.exit(1)

    elif "linux" in current_os or "darwin" in current_os:
        script_name = "build.sh"
        if os.path.exists(script_name):
            try:
                if not os.access(script_name, os.X_OK):
                    subprocess.run(["chmod", "+x", script_name], check=True)

                subprocess.run(["./" + script_name], check=True, shell=True)
            except subprocess.CalledProcessError as e:
                print(f"[!] Error executing {script_name}")
                sys.exit(e.returncode)
        else:
            print(f"[!] Error: {script_name} not found.")
            sys.exit(1)
    else:
        print(f"[!] Unsupported OS: {platform.system()}")
        sys.exit(1)


if __name__ == "__main__":
    build_project()
