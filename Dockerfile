FROM rust:1.37.0

COPY . .

#RUN cargo install --path .

CMD ["bin/child_issue"]

#COPY LICENSE README.md /
#
#COPY entrypoint.sh /entrypoint.sh
#
#ENTRYPOINT ["/entrypoint.sh"]
