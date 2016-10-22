#!/usr/bin/env python3
import argparse
import os
import sys
import fileinput
from yaml import load, dump

if __name__=="__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('files',metavar='video_file',nargs='*')
    args = parser.parse_args()

    for file in args.files:
        yamlfilename = os.path.splitext(file)[0] + ".yaml"
        try:
            f = open(yamlfilename,'ab+');
            f.seek(0);
            yaml_contents_str = f.read();
            yaml_contents = load(yaml_contents_str);
            print(yaml_contents)
            if yaml_contents is None:
                yaml_contents = {};
            yaml_contents['video_path'] = os.path.split(file)[1]
            try:
                video_duration = input("Longueur en secondes de la vidéo : ");
                yaml_contents['video_duration'] = int(1000.0 * float(video_duration))
            except KeyboardInterrupt:
                f.close();
                sys.exit(0);
            except EOFError:
                pass 
            f.truncate(0);
            f.write(dump(yaml_contents, encoding='utf-8', default_flow_style=False));
            f.close();
        except IOError:
            print("file '"+yamlfilename+"' couldnt be opened")

