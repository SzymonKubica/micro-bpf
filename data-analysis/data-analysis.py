import sys
import json
import csv
from collections import defaultdict


def process_data(file_name: str):
    with open(file_name, "r") as f:
        data = json.load(f)
        pretty_json = json.dumps(data, indent=4)
        print(pretty_json)


def process_fletcher16_320B():
    results_directory = "benchmark-results-raw"
    result_files = [
        "femtocontainers-header-fletcher-results.json",
        "extended-header-fletcher-results.json",
        "jit-fletcher-results.json",
        "native-fletcher-results.json",
    ]

    metrics = [
        "load_time",
        "verification_time",
        "execution_time",
        "total_time",
        "program_size",
    ]

    results_per_metric = defaultdict(lambda: {})
    for m in metrics:
        for file in result_files:
            file_name = f"{results_directory}/{file}"
            with open(file_name, "r") as f:
                data = json.load(f)
                vm_kind = file.replace("-fletcher-results.json", "")
                results_per_metric[m][vm_kind] = data["320"][m] if m in data["320"].keys() else 0


    for (metric, results) in results_per_metric.items():
        metric_str = metric.replace("_", "-")
        file_name = f"benchmark-results-processed/{metric_str}-results.csv"
        with open(file_name, "w") as f:
            writer = csv.DictWriter(f, fieldnames=["vm_kind", metric])
            writer.writeheader()
            for result in sorted(results.items(), key=lambda x: x[0]):
                writer.writerow({"vm_kind": result[0], metric: result[1]})



if __name__ == "__main__":
    # if len(sys.argv) < 2:
    #    print(f"Usage: python {sys.argv[0]} <file_name>")
    #    sys.exit(1)
    # file_name = sys.argv[1]
    # process_data(file_name)
    process_fletcher16_320B()
