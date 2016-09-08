#!/usr/bin/env python3
import argparse
import os

if __name__=="__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('files',metavar='video_file',nargs='*')
    args = parser.parse_args()

    for file in args.files:
        yamlfilename = os.path.splitext(file)[0] + ".yaml"
        try:
            f = open(yamlfilename,'x');
            f.write("video_path: \""+os.path.split(file)[1]+"\"")
            f.close()
        except FileExistsError:
            print("file '"+yamlfilename+"' already exists")
        except:
            print("file '"+yamlfilename+"' couldnt be opened")

