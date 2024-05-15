import sys
import json

def process_data(file_name: str):
    with open(file_name, 'r') as f:
        data = json.load(f)
        pretty_json = json.dumps(data, indent=4)
        print(pretty_json)



if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: python {sys.argv[0]} <file_name>")
        sys.exit(1)
    file_name = sys.argv[1]
    process_data(file_name)
