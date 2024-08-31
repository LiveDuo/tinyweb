import { ExternRef } from 'externref_polyfill';

/**
 * @typedef {Object} JSWasmHandlerContext
 * @property {Array<(...args: unknown[]) => number>} functions
 * @property {TextDecoder} utf8dec
 * @property {TextEncoder} utf8enc
 * @property {TextDecoder} utf16dec
 * @property {function(unknown): bigint} storeObject
 * @property {function(bigint): void} releaseObject
 * @property {WebAssembly.WebAssemblyInstantiatedSource} [module]
 * @property {function(number, number): string} readUtf8FromMemory
 * @property {function(number, number): string} readUtf16FromMemory
 * @property {function(): Uint8Array} getMemory
 * @property {function(number): [number,number]} createAllocation
 * @property {function(string): number} writeUtf8ToMemory
 * @property {function(ArrayBuffer): number} writeArrayBufferToMemory
 * @property {function(number, number): Uint8Array} readUint8ArrayFromMemory
 * @property {function(bigint): unknown} getObject
 * @property {function(number, number): unknown[]} readParameters
 */

/**
 * JsWasm environment handler
 */
const JsWasm = {
  /**
   * Creates a WebAssembly environment with necessary imports and context.
   * @returns {[Object, JSWasmHandlerContext]} - The module imports and context.
   */
  createEnvironment() {
    ExternRef.create(undefined);
    ExternRef.create(null);
    ExternRef.create(self);
    ExternRef.create(typeof document != "undefined" ? document : null);
    ExternRef.create(typeof document != "undefined" ? document.body : null);

    // 0 is reserved for undefined
    // 1 is reserved for null
    // 2 is reserved for self
    // 3 is reserved for document
    // 4 is reserved for document.body

    const context = {
      functions: [
        function () {
          debugger;
          return 0;
        },
      ],
      utf8dec: new TextDecoder("utf-8"),
      utf8enc: new TextEncoder(),
      utf16dec: new TextDecoder("utf-16"),
      readUtf8FromMemory: function (start, len) {
        const text = this.utf8dec.decode(
          this.getMemory().subarray(start, start + len)
        );
        return text;
      },
      createAllocation: function (size) {
        if (!this.module) {
          throw new Error("module not set");
        }
        const allocationId = this.module.instance.exports.create_allocation(size);
        const allocationPtr = this.module.instance.exports.allocation_ptr(allocationId);
        return [allocationId, allocationPtr];
      },
      writeUtf8ToMemory: function (str) {
        const bytes = this.utf8enc.encode(str);
        const len = bytes.length;
        const [id, start] = this.createAllocation(len);
        this.getMemory().set(bytes, start);
        return id;
      },
      writeArrayBufferToMemory: function (ab) {
        const bytes = new Uint8Array(ab);
        const len = bytes.length;
        const [id, start] = this.createAllocation(len);
        this.getMemory().set(bytes, start);
        return id;
      },
      readUtf16FromMemory: function (start, len) {
        const text = this.utf16dec.decode(
          this.getMemory().subarray(start, start + len)
        );
        return text;
      },
      readUint8ArrayFromMemory(start, length) {
        if (!this.module) {
          throw new Error("module not set");
        }
        const b = this.getMemory().slice(start, start + length);
        return new Uint8Array(b);
      },
      storeObject: function (obj) {
        return ExternRef.create(obj);
      },
      getObject: function (handle) {
        return ExternRef.load(handle);
      },
      releaseObject: function (handle) {
        // Don't release our fixed references
        if (handle <= 4n) {
          return;
        }
        ExternRef.delete(handle);
      },
      getMemory: function () {
        if (!this.module) {
          throw new Error("module not set");
        }
        return new Uint8Array(
          this.module.instance.exports.memory.buffer
        );
      },
      readParameters: function (start, length) {
        // Get bytes of parameters out of wasm module
        const parameters = this.readUint8ArrayFromMemory(start, length);
        // Convert bytes to array of values  
        // Assuming each parameter is preceded by a 32-bit integer indicating its type
        // 0 = undefined
        // 1 = null
        // 2 = float-64
        // 3 = bigint
        // 4 = string (followed by 32-bit start and size of string in memory)
        // 5 = extern ref
        // 6 = array of float-64 (followed by 32-bit start and size of string in memory)
        // 7 = true
        // 8 = false
        
        const values = [];
        let i = 0;
        while (i < parameters.length) {
          const type = parameters[i];
          i++;
          switch (type) {
            case 0:
              values.push(undefined);
              break;
            case 1:
              values.push(null);  
              break;
            case 2:
              values.push(new DataView(parameters.buffer).getFloat64(i, true));
              i += 8;
              break;
            case 3:
              values.push(new DataView(parameters.buffer).getBigInt64(i, true));
              i += 8;
              break;
            case 4: {
              const start = new DataView(parameters.buffer).getInt32(i, true);
              i += 4;
              const len = new DataView(parameters.buffer).getInt32(i, true);
              i += 4;
              values.push(
                context.readUtf8FromMemory(start, len)
              );
              break;
            }
            case 5: {
              const handle = new DataView(parameters.buffer).getBigInt64(i, true);
              values.push(context.getObject(handle));
              i += 8;
              break;
            }
            case 6: {
              const start = new DataView(parameters.buffer).getInt32(i, true);
              i += 4;
              const len = new DataView(parameters.buffer).getInt32(i, true);
              i += 4;
              const memory = context.getMemory();
              const slice = memory.buffer.slice(start, start + len * 4);
              const array = new Float32Array(slice);
              values.push(array);
              break;
            }
            case 7:
              values.push(true);
              break;
            case 8:
              values.push(false);  
              break;
            case 9: {
              const start = new DataView(parameters.buffer).getInt32(i, true);
              i += 4;
              const len = new DataView(parameters.buffer).getInt32(i, true);
              i += 4;
              const memory = context.getMemory();
              const slice = memory.buffer.slice(start, start + len * 8);
              const array = new Float64Array(slice);
              values.push(array);
              break;
            }
            case 10: {
              const start = new DataView(parameters.buffer).getInt32(i, true);
              i += 4;
              const len = new DataView(parameters.buffer).getInt32(i, true);
              i += 4;
              const memory = context.getMemory();
              const slice = memory.buffer.slice(start, start + len * 4);
              const array = new Uint32Array(slice);
              values.push(array);
              break;
            }
            default:
              throw new Error("unknown parameter type");
          }
        }
        return values;
      }
    };
    return [{
      abort() {
        throw new Error("WebAssembly module aborted");
      },
      externref_drop(obj) {
        context.releaseObject(obj);
      },
      js_register_function(start, len, utfByteLen) {
        let functionBody;
        if (utfByteLen === 16) {
          functionBody = context.readUtf16FromMemory(start, len);
        } else {
          functionBody = context.readUtf8FromMemory(start, len);
        }
        const id = context.functions.length;
        context.functions.push(
          Function(`"use strict";return(${functionBody})`)()
        );
        return id;
      },
      js_invoke_function(
        funcHandle,
        parametersStart,
        parametersLength
      ) {
        const values = context.readParameters(parametersStart, parametersLength);
        
        return context.functions[funcHandle].call(
          context,
          ...values 
        );
      },
      js_invoke_function_and_return_object(
        funcHandle,
        parametersStart,
        parametersLength
      ) {
        const values = context.readParameters(parametersStart, parametersLength);
        const result = context.functions[funcHandle].call(
          context,
          ...values 
        );
        if(result === undefined || result === null) {
          throw new Error("js_invoke_function_and_return_object returned undefined or null while trying to return an object");
        }
        return context.storeObject(result);
      },
      js_invoke_function_and_return_bool(
        funcHandle,
        parametersStart,
        parametersLength
      ) {
        const values = context.readParameters(parametersStart, parametersLength);
        const result = context.functions[funcHandle].call(
          context,
          ...values
        );
        return result ? 1 : 0;
      },
      js_invoke_function_and_return_bigint(
        funcHandle,
        parametersStart,
        parametersLength
      ) {
        const values = context.readParameters(parametersStart, parametersLength);
        const result = context.functions[funcHandle].call(
          context,
          ...values 
        );
        return result;
      },
      js_invoke_function_and_return_string(
        funcHandle,
        parametersStart,
        parametersLength
      ) {
        const values = context.readParameters(parametersStart, parametersLength);
        const result = context.functions[funcHandle].call(
          context,
          ...values 
        );

        if(result === undefined || result === null) {
          throw new Error("js_invoke_function_and_return_string returned undefined or null while trying to retrieve string.");
        }
        return context.writeUtf8ToMemory(result);
      },
      js_invoke_function_and_return_array_buffer(
        funcHandle,
        parametersStart,
        parametersLength
      ) {
        const values = context.readParameters(parametersStart, parametersLength);
        const result = context.functions[funcHandle].call(
          context,
          ...values 
        );

        if(result === undefined || result === null) {
          throw new Error("js_invoke_function_and_return_array_buffer returned undefined or null while trying to retrieve arraybuffer.");
        }
        return context.writeArrayBufferToMemory(result);
      }
    }, context];
  },

  /**
   * Loads and runs a WebAssembly module from the provided URL.
   * @param {string} wasmURL - The URL of the WebAssembly module.
   */
  async loadAndRunWasm(wasmURL) {
    const context = await this.load(wasmURL);
    (context.module.instance.exports.main)();
  },

  /**
   * Loads a WebAssembly module from the provided URL.
   * @param {string} wasmURL - The URL of the WebAssembly module.
   * @returns {Promise<JSWasmHandlerContext>} - The context of the loaded module.
   */
  async load(wasmURL) {
    const [env, context] = JsWasm.createEnvironment();
    const response = await fetch(wasmURL);
    const bytes = await response.arrayBuffer();
    const module = await WebAssembly.instantiate(bytes, {
      env,
    });
    context.module = module;
    return context;
  },
};

document.addEventListener("DOMContentLoaded", function () {
  const wasmScripts = document.querySelectorAll(
    "script[type='application/wasm']"
  );
  for (let i = 0; i < wasmScripts.length; i++) {
    const src = wasmScripts[i].src;
    if (src) {
      JsWasm.loadAndRunWasm(src);
    } else {
      console.error("Script tag must have 'src' property.");
    }
  }
});
