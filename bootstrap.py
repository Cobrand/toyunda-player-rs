#!/bin/env python3
import urllib.request
import errno    
import os

def urlretrieve(url,file_name):
    outfile = open(file_name, 'w+b')
    hdr = {'Accept':'*/*','User-Agent': 'curl'}
    request = urllib.request.Request(url,headers=hdr)
    try:
        response = urllib.request.urlopen(request)
        data = response.read()
        outfile.write(data)
        print("downloaded '"+url+"'")
    except:
        print("failed to download '"+url+"'")

def mkdir_p(path):
    try:
        os.makedirs(path)
    except OSError as exc:  # Python >2.5
        if exc.errno == errno.EEXIST and os.path.isdir(path):
            pass
        else:
            raise

if __name__ == "__main__":
    mkdir_p("web/libs");

    # download the js libs
    urlretrieve("https://rc.vuejs.org/js/vue.js","web/libs/vue.js");
    urlretrieve("https://cdn.jsdelivr.net/sweetalert2/4.2.6/sweetalert2.js","web/libs/swal.js");
    urlretrieve("https://cdn.jsdelivr.net/sweetalert2/4.2.6/sweetalert2.css","web/libs/swal.css");
