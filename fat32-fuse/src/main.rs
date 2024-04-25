use clap::{Arg, App};
use std::fs::{OpenOptions, read_dir,File,DirEntry};
use std::io::{self, Read};
use fscommon::BufStream;
use fatfs::{format_volume,FormatVolumeOptions,StdIoWrapper,FsOptions,FileSystem,Write, FatType};

fn fat32_packup(){
    //cargo run --后面再使用这些参数
    let matches = App::new("Fat32 packer")
        //源文件/目录
        .arg(Arg::with_name("source")
            .short("s")
            .long("source")
            .takes_value(true)
            .help("Executable source dir(with backslash)")
        )
        //目标路径（貌似没什么用）
        .arg(Arg::with_name("target")
            .short("t")
            .long("target")
            .takes_value(true)
            .help("Executable target dir(with backslash)")
        )
        //输出镜像名称
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .takes_value(true)
            .help("img output"))
        //当只想测评某个样例时使用，所以不需要takes_values
        .arg(Arg::with_name("bin")
            .short("b")
            .long("bin")
            .help("use when source is binary"))        
        .get_matches();
    let src_path = matches.value_of("source").unwrap();
    let target_path = matches.value_of("target").unwrap();
    let output = matches.value_of("output").unwrap();
    println!("src_path = {}\ntarget_path = {}\noutput_file={}", src_path, target_path,output);

    //创建一个镜像
    create_img(output).unwrap();

    //打开镜像文件,参考fatfs中的write.rs
    let imgfile = OpenOptions::new()
        .read(true)
        .write(true)
        .open(output)
        .unwrap();
    let buf_stream = BufStream::new(imgfile);
    let options = FsOptions::new().update_accessed_date(true);
    let fs = FileSystem::new(buf_stream,options).unwrap();
    let root = fs.root_dir();

    //读取文件(还没用上bin)
    let apps = get_files(src_path);

    //将文件写入镜像
    for app in apps{
        //子目录
        if app.ends_with("/"){
            //println!("user dir: {}",app.as_str());
            root.create_dir(app.as_str()).unwrap();
        }else{
            //println!("user app:{}",app.as_str());
            let mut host_file = File::open(format!("{}{}",target_path,app)).unwrap();
            let mut data:Vec<u8> = Vec::new();
            host_file.read_to_end(&mut data).unwrap();
            let mut inode = root.create_file(app.as_str()).unwrap();
            inode.write_all(data.as_slice()).unwrap();
        }
    }

    for entry in root.iter(){
        let file = entry.unwrap();
        println!("{}",file.file_name());
        if file.is_dir(){
            for inner in root.open_dir(file.file_name()
            .as_str())
            .unwrap()
            .iter(){
                let inner_name=inner.unwrap().file_name().to_string();
                if inner_name != String::from(".") && inner_name != String::from(".."){
                    println!("{}",inner_name);
            }
        } 
    }
    }

}

//
fn create_img(output:&str)->io::Result<()>{
    let img = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&output)
        .unwrap();
    //设置大小
    img.set_len(512*2048*512).unwrap();
    //模仿fatfs中的mkfatfs.rs，将该镜像格式化为fat32格式
    let buf_file = BufStream::new(img);
    let form = FormatVolumeOptions::new();
    let form = form.fat_type(FatType::Fat32);
    format_volume(&mut StdIoWrapper::from(buf_file), form)
        .unwrap();
    
    Ok(())
}


//获取文件名称(包括子目录下的)
fn get_files(path:& str)->Vec<String>{
    let mut names: Vec<String> = vec![];
    for entry in read_dir(path).unwrap(){
        let file = entry.unwrap();
        let file_name = file.file_name().into_string().unwrap();
        //碰上子目录了，需要递归读取
        if file.path().is_dir(){
            //保存目录项
            names.push(format!("{}/",file_name));
            for inner_entry in read_dir(file.path()).unwrap(){
                traverse(
                    inner_entry.unwrap(),
                    format!("{}/",file_name),
                    &mut names
                );
            }

        }else{
            names.push(format!("{}",file_name));
        }
    }
    names
}

//递归读取
fn traverse(file:DirEntry,target_dir:String,names:&mut Vec<String>){
    let file_name = file.file_name().into_string().unwrap();
    if file.path().is_dir() {
        names.push(format!("{}{}/", target_dir, file_name));
        for inner_entry in read_dir(file.path()).unwrap() {
            traverse(
                inner_entry.unwrap(),
                format!("{}{}/", target_dir, file_name),
                names,
            );
        }
    } else {
        names.push(format!("{}{}", target_dir, file_name));
    }
}
fn main() {
    fat32_packup();
}
