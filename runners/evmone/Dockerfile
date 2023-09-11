FROM ubuntu:latest

WORKDIR /runner/
COPY CMakeLists.txt runner.cpp ./
COPY cmake ./cmake

RUN cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
RUN cmake --build build --parallel

ENTRYPOINT [ "build/runner-evmone" ]