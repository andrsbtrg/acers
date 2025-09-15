from acers import clash_detection


with open("./examples/set_a.txt", "r") as file:
    set_a = file.read()

with open("./examples/set_b.txt", "r") as file:
    set_b = file.read()

results = clash_detection(set_a, set_b, 0.0)

print("results:", results)
for result in results:
    print(result.dist)
