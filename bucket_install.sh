if type php; then
    cp inapi.so $(php -r "echo ini_get('extension_dir');")
fi

# Create module ini file
if [ -d /etc/php.d ]; then
    echo 'extension=inapi.so' > /etc/php.d/inapi.ini
elif [ -d /etc/php5 ]; then
    echo 'extension=inapi.so' > /etc/php5/mods-available/inapi.ini
    ln -s /etc/php5/mods-available/inapi.ini /etc/php5/apache2/conf.d/20-inapi.ini
    ln -s /etc/php5/mods-available/inapi.ini /etc/php5/cli/conf.d/20-inapi.ini
elif [ -f /usr/local/etc/php/extensions.ini ]; then
    echo 'extension=inapi.so' >> /usr/local/etc/php/extensions.ini
fi
