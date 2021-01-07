FROM debian:buster-slim

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && apt-get install htop \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC

WORKDIR /

COPY ["target/release/authentication_microservice", "rust/authentication_microservice"]

EXPOSE 8080
RUN chmod +x rust/authentication_microservice
CMD ["./rust/authentication_microservice"]