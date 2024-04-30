ARG BASE_IMAGE

FROM --platform=${BUILDPLATFORM} alpine as artifact
COPY artifact/ /artifact/
ARG TARGET_ENV
ARG TARGETARCH
ENV TARGETARCH=${TARGETARCH}
ENV TARGET_ENV=${TARGET_ENV}
RUN mv /artifact/${TARGETARCH}-${TARGET_ENV}/nghe /artifact/nghe
RUN chmod +x /artifact/nghe
RUN apk add --update --no-cache ca-certificates
RUN update-ca-certificates

FROM ${BASE_IMAGE}
COPY --from=artifact /artifact/nghe /app/nghe
COPY --from=artifact /artifact/frontend/ /app/frontend/
COPY --from=artifact /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
ENV NGHE_SERVER__HOST="::"
ENV NGHE_SERVER__FRONTEND_DIR="/app/frontend/"
ENTRYPOINT [ "/app/nghe" ]
