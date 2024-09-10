# Header files directories
RLBOX_INCLUDE = {{for include in rlbox.include}}"{include}" {{endfor}}

# WASI Runtime (.c) files
WASI_RUNTIME_FILES = {{for rt_file in rlbox.wasi_runtime_files}}"{rt_file}" {{endfor}}

# Binaries used for compilation
WASI_CLANG="{rlbox.wasi_clang}"
WASM2C="{rlbox.wasm2c}"

all: {env.package_directory}/lib{name}_sandbox.so

# Step 2: use wasm2c to convert our WASM to a C implementation of WASM we can link with our application
{name}.wasm.c: {env.package_directory}/wasm_target/wasm32-rlbox/release/{name}.wasm
	$(WASM2C) {env.package_directory}/wasm_target/wasm32-rlbox/release/{name}.wasm -o {name}.wasm.c

# Step 3: generating .o files.
{name}.wasm.o: {name}.wasm.c {name}.wasm.h wasi_rt.aux.c
	$(CC) -DALLOW_SANDBOX_PRINTING -O3 -fPIC -c $(WASI_RUNTIME_FILES) $(patsubst %, -I%, $(RLBOX_INCLUDE)) {name}.wasm.c wasi_rt.aux.c
	# To dissallow printing; use below instead
	# $(CC) -O3 -fPIC -c $(WASI_RUNTIME_FILES) $(patsubst %, -I%, $(RLBOX_INCLUDE)) {name}.wasm.c wasi_rt.aux.c

# Step 4: linking our RLBox wrapper functions with our library of functions and RLBox into a shared library
{env.package_directory}/lib{name}_sandbox.so: {name}.wasm.o wrapper.cpp wrapper.h
	$(CXX) -O3 -std=c++17 -rdynamic -shared -fPIC -o {env.package_directory}/lib{name}_sandbox.so wrapper.cpp $(patsubst %, -I%, $(RLBOX_INCLUDE)) *.o -lpthread
