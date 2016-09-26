import shutil
import distutils.dir_util
import os
import os.path

def mkdir_p(path):
    try:
        os.makedirs(path)
    except OSError as exc:  # Python >2.5
        if exc.errno == errno.EEXIST and os.path.isdir(path):
            pass
        else:
            raise

if __name__=="__main__":
    out_dir = os.environ["OUT_DIR"]
    out_dir = os.path.join(out_dir,"../../../")
    out_web = os.path.join(out_dir,"web/")
    mkdir_p(out_web)
    shutil.rmtree(out_web)
    distutils.dir_util.copy_tree("web/",out_web)
    shutil.copyfile("logo_toyunda.png",os.path.join(out_dir,"logo_toyunda.png"))
