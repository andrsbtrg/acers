from acers import clash_detection


with open("set_a.txt", "r") as file:
    set_a = file.read()

with open("set_b.txt", "r") as file:
    set_b = file.read()

results = clash_detection(set_a, set_b)

print(results)
