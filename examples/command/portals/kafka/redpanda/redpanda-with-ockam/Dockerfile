FROM docker.redpanda.com/redpandadata/redpanda:v23.1.13@sha256:5cca0f6ab9f6f932f443182d5587d835d23f6c66ad85127cfe2ccd2c237ff222

RUN curl --proto '=https' --tlsv1.2 -sSf https://install.command.ockam.io | bash
ENV PATH="/var/lib/redpanda/.ockam/bin:$PATH"
COPY --chown=redpanda:redpanda ./ockam.sh /var/lib/redpanda/ockam.sh
RUN chmod +x /var/lib/redpanda/ockam.sh

ENTRYPOINT ["/var/lib/redpanda/ockam.sh"]
CMD ["redpanda", "start", "--kafka-addr", "internal://0.0.0.0:9092,external://0.0.0.0:19092", \
    "--advertise-kafka-addr internal://redpanda-0:9092", \
    "--pandaproxy-addr internal://0.0.0.0:8082", \
    "--advertise-pandaproxy-addr internal://redpanda-0:8082", \
    "--schema-registry-addr internal://0.0.0.0:8081", \
    "--smp 1", \
    "--memory 1G", \
    "--mode dev-container", \
    "--default-log-level=debug"]
