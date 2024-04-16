ARG BASE_IMAGE

FROM --platform=${BUILDPLATFORM} busybox as artifact
COPY artifact/ /artifact/
ARG TARGET_ENV
ARG TARGETARCH
ENV TARGETARCH=${TARGETARCH}
ENV TARGET_ENV=${TARGET_ENV}
RUN mv /artifact/${TARGETARCH}-${TARGET_ENV}/nghe /artifact/nghe
RUN chmod +x /artifact/nghe

FROM ${BASE_IMAGE}
COPY --from=artifact /artifact/nghe /app/nghe
COPY --from=artifact /artifact/frontend/ /app/frontend/
ENV NGHE_SERVER__HOST=::
ENV NGHE_SERVER__FRONTEND_DIR="/app/frontend/"
ENTRYPOINT [ "/app/nghe" ]
