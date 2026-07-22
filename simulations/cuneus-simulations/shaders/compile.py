# Simple utility to add some wesl features to wgsl
# I want to add include! in wgsl, and I can't make wesl work nicely (crashes even though I have hot reloading + not pretty errors)

from sys import argv
from watchdog.observers import Observer
from watchdog.events import *
import os,sys,time
global ALL
ALL = [False, ]
def main():
    if len(argv) < 2:
        print("Usage: python compile.py <path_to_wgsl_file>")
        return

    wgsl_file = argv[1]
    if len(argv)>=2 and argv[2] == '--all':
        ALL[0] = True
        print("Directory mode")
        for file in os.listdir(wgsl_file):
            if file.endswith('.wgsl'):
                compile_wgsl(os.path.join(wgsl_file, file))
    else:
        compile_wgsl(wgsl_file)
    dirname = os.path.dirname(wgsl_file)
    if dirname.strip() == '':
        dirname = '.'
    print(f"Watching {dirname} for changes...")
    event_handler = FileSystemEventHandler()
    observer = Observer()
    observer.schedule(event_handler, dirname, recursive=True)
    observer.start()
    event_handler.on_modified = on_modified(wgsl_file)
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
    observer.join()

def on_modified(wgsl_file):
    def on_modified_inner(event: FileModifiedEvent):
        # print(event.src_path)
        if (event.src_path.endswith('-compiled.wgsl')) or (not event.src_path.endswith('.wgsl')) or (event.is_directory):return
        # print(event.event_type, event.is_synthetic, event)
        try:
            if ALL[0]:
                for file in os.listdir(wgsl_file):
                    if str(file).endswith('.wgsl'):
                        compile_wgsl(os.path.join(wgsl_file, file))
                compile_wgsl(event.src_path)
            else: compile_wgsl(wgsl_file)
        except IsADirectoryError:pass
        except Exception as e:
            print(f"Error compiling {wgsl_file}: {e}")
    return on_modified_inner


def compile_wgsl(file: str):
    print(f"Compiling {file}"),
    if not os.path.exists(file):
        print(f"File {file} does not exist.")
        sys.exit(1)

    # Put in compiled directory
    output_file = os.path.join(os.path.dirname(file), 'compiled', os.path.basename(file).replace('.wgsl', '-compiled.wgsl'))
    formatted_content = format_doc(file)

    with open(output_file, 'w') as f:
        f.write(formatted_content)

    # print(f"Compiled {file} to {output_file}")


def format_doc(path: str) -> str:
    if not os.path.exists(path):
        print(f"File {path} does not exist.")
        sys.exit(1)

    # print(f"Compiling {path}...")
    with open(os.path.abspath(path), 'r') as file:
        content = file.read()
    lines = content.splitlines()
    new_lines = []
    for line in lines:
        if line.startswith("include!(\""):
            include_path = line[10:].strip().strip("\"").strip("\");")
            # print(f"  -> {include_path}")
            # Append current path
            new_lines.append(f"// ----- START: {include_path} --------")
            new_lines.append(format_doc(os.path.join(os.path.dirname(path), include_path)))
            new_lines.append(f"// ----- END: {include_path} --------")
        else:
            new_lines.append(line)
    return "\n".join(new_lines)


if __name__ == "__main__":
    main()
