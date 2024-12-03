import time

def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

def run_benchmark():
    # Test 1: Fibonacci
    start = time.time()
    fib_result = fibonacci(8)
    fib_time = time.time() - start
    print(f"Python Fibonacci(8): {fib_result}, Time: {fib_time:.4f} seconds")
if __name__ == "__main__":
    run_benchmark()
