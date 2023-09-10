FROM python:latest

# Need Cargo for revm backing pyrevm
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN curl -sSL https://install.python-poetry.org | python3 -
ENV PATH="/root/.local/bin:${PATH}"

RUN echo ${PATH}

WORKDIR /runner/
COPY pyproject.toml runner.py ./

RUN cargo
RUN poetry install

ENTRYPOINT [ "poetry", "run", "python", "runner.py" ]