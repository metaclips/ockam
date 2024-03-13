# This dockerfile builds an image that contains bash and ockam command.
#
# It also copies a bash script called run_ockam.sh from its build directory
# into the image being built and uses that script as entrypoint to containers
# that are run using this image.
#
# The run_ockam.sh script is used to set up and start an ockam node.
#
# Read bank_corp/run_ockam.sh and analysis_corp/run_ockam.sh to understand
# how each node is set up.

FROM ghcr.io/build-trust/ockam@sha256:e32ac11f43b16e9b2d889088405c0da54e590726236669f09465b3bb205c40c9 as builder

FROM cgr.dev/chainguard/bash@sha256:e4e1f63802396154706a44017f23bd3bfba4f8684374c4c981ba7567636a948e
COPY --from=builder /ockam /usr/local/bin/ockam

COPY run_ockam.sh /run_ockam.sh
RUN chmod +x /run_ockam.sh
ENTRYPOINT ["/run_ockam.sh"]
