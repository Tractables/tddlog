"""Build and test the C package: the committed header against cbindgen's, the
CMake build and its tests, and the example built against the installed
package, moved to another prefix, through CMake and through pkg-config."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent
BUILD = ROOT / "build"


def run(*args, **kwargs):
    print("+", " ".join(str(arg) for arg in args), flush=True)
    return subprocess.run([str(arg) for arg in args], check=True, **kwargs)


def check_header(write):
    """Build the library, whose build script writes the header cbindgen
    generates, and compare it with the committed one."""
    result = run("cargo", "build", "--release", "--manifest-path", ROOT / "Cargo.toml",
                 "--target-dir", BUILD / "cargo", "--message-format=json",
                 stdout=subprocess.PIPE, text=True)
    headers = []
    for line in result.stdout.splitlines():
        item = json.loads(line)
        if item.get("reason") == "build-script-executed":
            header = Path(item["out_dir"]) / "tddlog.h"
            if header.is_file():
                headers.append(header)
    if len(headers) != 1:
        raise SystemExit(f"expected one generated tddlog.h, found {len(headers)}")
    generated = headers[0].read_text(encoding="utf-8")
    committed = ROOT / "include" / "tddlog.h"
    if write:
        committed.write_text(generated, encoding="utf-8")
    elif committed.read_text(encoding="utf-8") != generated:
        raise SystemExit("include/tddlog.h is stale; run python bindings/c/check.py --write-header")


def check_installation(package):
    """Build and run the example against the installed package, moved to
    another prefix and with the build's cargo output out of the way."""
    with tempfile.TemporaryDirectory(prefix="installed-", dir=BUILD) as temporary:
        temporary = Path(temporary)
        prefix = temporary / "prefix"
        hidden = temporary / "cargo-hidden"
        shutil.copytree(package, prefix, symlinks=True)
        (BUILD / "cargo").rename(hidden)
        try:
            libraries = prefix / "lib"
            environment = dict(os.environ)
            for variable in ("LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH"):
                environment[variable] = str(libraries) + os.pathsep + environment.get(variable, "")

            consumer = temporary / "consumer"
            consumer.mkdir()
            shutil.copy(ROOT / "examples" / "alarm.c", consumer / "alarm.c")
            (consumer / "CMakeLists.txt").write_text("""cmake_minimum_required(VERSION 3.16)
project(installed_consumer LANGUAGES C)
find_package(tddlog CONFIG REQUIRED)
add_executable(alarm alarm.c)
target_link_libraries(alarm PRIVATE tddlog::tddlog m)
set_target_properties(alarm PROPERTIES RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
  RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CMAKE_BINARY_DIR}/bin")
""", encoding="utf-8")
            build = temporary / "consumer-build"
            run("cmake", "-S", consumer, "-B", build, f"-DCMAKE_PREFIX_PATH={prefix}",
                "-DCMAKE_BUILD_TYPE=Release")
            run("cmake", "--build", build, "--config", "Release")
            run(build / "bin" / "alarm", env=environment)

            pc = dict(environment, PKG_CONFIG_PATH=str(prefix / "lib" / "pkgconfig"))
            flags = run("pkg-config", "--cflags", "--libs", "tddlog", env=pc,
                        stdout=subprocess.PIPE, text=True).stdout.split()
            program = temporary / "alarm-pkg-config"
            run(os.environ.get("CC", "cc"), "-std=c11", "-Wall", "-Wextra", "-Werror",
                consumer / "alarm.c", *flags, "-lm", "-o", program)
            run(program, env=environment)
        finally:
            hidden.rename(BUILD / "cargo")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write-header", action="store_true",
                        help="replace the committed header with the generated one")
    args = parser.parse_args()
    for tool in ("cargo", "cmake", "ctest", "pkg-config"):
        if shutil.which(tool) is None:
            raise SystemExit(f"check.py needs {tool} on PATH")
    check_header(args.write_header)
    run("cmake", "-S", ROOT, "-B", BUILD, "-DCMAKE_BUILD_TYPE=Release", "-DCMAKE_INSTALL_LIBDIR=lib")
    run("cmake", "--build", BUILD, "--config", "Release")
    run("ctest", "--test-dir", BUILD, "-C", "Release", "--output-on-failure")
    package = BUILD / "package"
    shutil.rmtree(package, ignore_errors=True)
    run("cmake", "--install", BUILD, "--config", "Release", "--prefix", package)
    check_installation(package)
    print("all C checks passed")


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        sys.exit(f"{error.cmd[0]} failed with status {error.returncode}")
