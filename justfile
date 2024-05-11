build-win:
    docker build docker/windows-cross-compile/ -t rust_cross_compile/windows
    docker run --rm -v $(pwd):/app rust_cross_compile/windows
