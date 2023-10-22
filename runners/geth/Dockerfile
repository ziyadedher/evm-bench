FROM golang:latest

WORKDIR /runner/
COPY go.mod go.sum runner.go ./

RUN go get
RUN go install

ENTRYPOINT [ "runner-geth" ]