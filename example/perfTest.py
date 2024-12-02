import time

def calculate_sum_of_squares(n):
    total = 0
    for i in range(n):
        total += i * i
    return total

def run_performance_test():
    iterations = 10000
    
    # Record start time
    start_time = time.time()
    
    # Perform calculation
    result = calculate_sum_of_squares(iterations)
    
    # Record end time
    end_time = time.time()
    
    # Calculate duration
    duration = end_time - start_time
    
    print("=== Python Performance Test ===")
    print(f"Iterations: {iterations}")
    print(f"Result: {result}")
    print(f"Time taken: {duration:.4f} seconds")

if __name__ == "__main__":
    run_performance_test()
