PHP_ARG_WITH(inapi,
    [whether to enable the "inapi" extension],
    [  --with-inapi          Enable "inapi" extension support])

if test "$PHP_INAPI" != "no"; then
    SEARCH_FOR="libinapi"
    if test -r $PHP_INAPI/$SEARCH_FOR.so; then
      INAPI_LIB_DIR=$PHP_INAPI
      INAPI_INC_DIR=$PHP_INAPI/../include
    else
      if test -r $PHP_INAPI/$SEARCH_FOR.dylib; then
        INAPI_LIB_DIR=$PHP_INAPI
        INAPI_INC_DIR=$PHP_INAPI/../include
      else
        AC_MSG_CHECKING([for inapi files in default path])
        for p in /usr/local /usr; do
          for e in so dylib; do
            if test -r $p/lib/$SEARCH_FOR.$e; then
              INAPI_LIB_DIR=$p/lib
              INAPI_INC_DIR=$p/include
              AC_MSG_RESULT(found in $p)
            fi
          done
        done
      fi
    fi

    if test -z "$INAPI_LIB_DIR"; then
      AC_MSG_RESULT([not found])
      AC_MSG_ERROR([Please install the inapi Rust library])
    fi

    PHP_CHECK_LIBRARY(inapi, host_connect,
    [
        PHP_ADD_INCLUDE($INAPI_INC_DIR)
        PHP_ADD_LIBRARY_WITH_PATH(inapi, $INAPI_LIB_DIR, INAPI_SHARED_LIBADD)
        AC_DEFINE(HAVE_INAPI, 1, [whether host_connect function exists])
    ],[
        AC_MSG_ERROR([host_connect function not found in libinapi])
    ],[
        -L$INAPI_LIB_DIR -R$INAPI_LIB_DIR
    ])

    PHP_SUBST(INAPI_SHARED_LIBADD)
    PHP_NEW_EXTENSION(inapi, php_inapi.c wrappers/command.c wrappers/directory.c wrappers/host.c wrappers/package.c wrappers/service.c wrappers/file.c wrappers/template.c, $ext_shared)
fi
