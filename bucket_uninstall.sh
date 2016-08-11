PED=$(php -r "echo ini_get('extension_dir');")
rm -f "$PED/inapi.so"

if [ -d /etc/php.d ]; then
    rm -f /etc/php.d/inapi.ini
elif [ -d /etc/php5 ]; then
    rm -f /etc/php5/mods-available/inapi.ini /etc/php5/cli/conf.d/20-inapi.ini
elif [ -f /usr/local/etc/php/extensions.ini ]; then
    sed 's/extension=inapi.so//' </usr/local/etc/php/extensions.ini >extensions.ini.new
    mv extensions.ini.new /usr/local/etc/php/extensions.ini
fi
