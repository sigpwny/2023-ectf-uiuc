import random

def generate_id(length=32):
  return int("".join([str(random.randint(0, 9)) for i in range(length)]))
