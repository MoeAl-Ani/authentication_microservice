FROM rust:1.31
WORKDIR /

COPY ["target/release/authentication_microservice", "authentication_microservice"]

EXPOSE 8080

CMD authentication_microservice