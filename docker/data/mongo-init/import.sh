echo IMPORTING COLLECTION
shopt -s nullglob dotglob     # To include hidden files
files=(/backup/*.gz)
if [ ${#files[@]} -gt 0 ]; then
    mongorestore --gzip --archive=/backup
fi
echo DONE IMPORTING