import re
import sys
def update_base_address(file_path, new_address):
    try:
        with open(file_path, 'r') as f:
            content = f.read()
            # 使用正则表达式查找BASE_ADDRESS并替换
            content = re.sub(r'(BASE_ADDRESS\s*=\s*)0x[0-9a-fA-F]+', rf'\g<1>{new_address}', content)

        with open(file_path, 'w') as f:
            f.write(content)

        print(f"BASE_ADDRESS updated to {new_address} in {file_path}")
    except FileNotFoundError:
        print(f"File {file_path} not found.")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("py脚本中链接脚本基址设置有问题!")

    os = "src/linker.ld"
    user = "../user/src/linker.ld"
    qemu_os_address = "0x80200000"
    qemu_user_address = "0x80400000"
    k210_os_address = "0x80020000"
    k210_user_address = "0x80040000"
    if sys.argv[1] == "qemu":
        update_base_address(os, qemu_os_address)
        update_base_address(user, qemu_user_address)
    elif sys.argv[1] == "k210":
        update_base_address(os, k210_os_address)
        update_base_address(user, k210_user_address)
    else:
        print("BASE_ADDRESS设置有问题！")
