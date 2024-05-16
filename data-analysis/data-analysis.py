import sys
import json
import csv
from collections import defaultdict


def process_data(file_name: str):
    with open(file_name, "r") as f:
        data = json.load(f)
        pretty_json = json.dumps(data, indent=4)
        print(pretty_json)


def process_fletcher16(data_size: int):
    """
    Takes in the data size of the fletcher16 benchmark (between 80-2560B)
    and produces csv outputs for that datapoint.
    """
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

    # this array controlls the sorting of the fields in the output csv files
    platforms = ["native", "femtocontainers-header", "extended-header", "jit"]

    results_per_metric = defaultdict(lambda: {})
    for m in metrics:
        for file in result_files:
            file_name = f"{results_directory}/{file}"
            with open(file_name, "r") as f:
                data = json.load(f)
                vm_kind = file.replace("-fletcher-results.json", "")
                results_per_metric[m][vm_kind] = data[str(data_size)][m] if m in data[str(data_size)].keys() else 0


    for (metric, results) in results_per_metric.items():
        metric_str = metric.replace("_", "-")
        file_name = f"benchmark-results-processed/fletcher16-{data_size}-{metric_str}-results.csv"
        with open(file_name, "w") as f:
            writer = csv.DictWriter(f, fieldnames=["platform", metric])
            writer.writeheader()
            for platform in platforms:
                # in the native C case, we don't measure all metrics so we skip this entry in the CSV
                if platform == "native" and metric in ["load_time", "verification_time", "program_size"]:
                    continue
                # in case of native total time is the execution time
                if platform == "native" and metric == "total_time":
                  writer.writerow({"platform": platform, metric: results_per_metric["execution_time"][platform]})
                  continue
                writer.writerow({"platform": platform, metric: results[platform]})

            # We need to append this dummy row at the  end because that's how
            # the latex csv parser works
            writer.writerow({"platform": 0, metric: 0})




if __name__ == "__main__":
    # if len(sys.argv) < 2:
    #    print(f"Usage: python {sys.argv[0]} <file_name>")
    #    sys.exit(1)
    # file_name = sys.argv[1]
    # process_data(file_name)
    process_fletcher16(640)
