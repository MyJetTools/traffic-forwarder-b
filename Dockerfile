FROM rust:slim
COPY ./target/release/traffic-forwarder-b ./target/release/traffic-forwarder-b
COPY ./wwwroot ./wwwroot 
ENTRYPOINT ["./target/release/traffic-forwarder-b"]