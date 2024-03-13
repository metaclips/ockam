# This dockerfile builds an image that contains nodejs.
#
# It also copies a bash script called run_ockam.sh from its build directory
# into the image being built and uses that script as entrypoint to containers
# that are run using this image.
#
# The run_ockam.sh script is used to setup an ockam node.

FROM cgr.dev/chainguard/node@sha256:801bbe84e6f9be40ebcc7cbaedfd6bd6b9583157546359416dbf7bb037aea9ca
ENV NODE_ENV=production

WORKDIR /app

RUN npm install pg@8.11.3
COPY --chown=node:node app.js app.js
ENTRYPOINT [ "node", "app.js" ]
