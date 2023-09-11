FROM alpine:latest

RUN apk add build-base make cmake git

WORKDIR /runner/
COPY CMakeLists.txt runner.cpp ./
COPY cmake ./cmake

RUN cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
RUN cmake --build build --parallel

ENTRYPOINT [ "build/runner-evmone" ]