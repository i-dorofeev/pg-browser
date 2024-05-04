FROM postgres:15
COPY entry-point.sh /usr/local/bin
COPY docker-entrypoint-initdb.d/ /docker-entrypoint-initdb.d/
ENTRYPOINT ["entry-point.sh"]