FROM arm64v8/rust:1.75.0-slim-bookworm

WORKDIR /workspace

RUN apt update && apt install -y build-essential libgtk-4-dev libglib2.0-dev
