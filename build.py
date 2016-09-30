import shutil
import distutils.dir_util
import os
import os.path

if __name__=="__main__":
    out_dir = os.environ["OUT_DIR"]
    out_dir = os.path.join(out_dir,"../../../")
    out_web = os.path.join(out_dir,"web/")
    os.makedirs(out_web,exist_ok=True)
    shutil.rmtree(out_web)
    distutils.dir_util.copy_tree("web/",out_web)
    shutil.copyfile("logo_toyunda.png",os.path.join(out_dir,"logo_toyunda.png"))
