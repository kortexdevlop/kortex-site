// @ts-nocheck
// Generated from frozen workflow V1; no eval at runtime.
var __getOwnPropNames = Object.getOwnPropertyNames;
var __commonJS = (cb, mod) => function __require() {
  try {
    return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
  } catch (e) {
    throw mod = 0, e;
  }
};

// node_modules/.pnpm/ajv@8.20.0/node_modules/ajv/dist/runtime/ucs2length.js
var require_ucs2length = __commonJS({
  "node_modules/.pnpm/ajv@8.20.0/node_modules/ajv/dist/runtime/ucs2length.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    function ucs2length(str) {
      const len = str.length;
      let length = 0;
      let pos = 0;
      let value;
      while (pos < len) {
        length++;
        value = str.charCodeAt(pos++);
        if (value >= 55296 && value <= 56319 && pos < len) {
          value = str.charCodeAt(pos);
          if ((value & 64512) === 56320)
            pos++;
        }
      }
      return length;
    }
    exports.default = ucs2length;
    ucs2length.code = 'require("ajv/dist/runtime/ucs2length").default';
  }
});

// <stdin>
var workflow = validate20;
var func1 = require_ucs2length().default;
var pattern4 = new RegExp("^[A-Za-z][A-Za-z0-9_-]*$", "u");
var pattern6 = new RegExp("^[a-z0-9]+(?:-[a-z0-9]+)*\\.[a-z][a-z0-9_-]*$", "u");
var pattern7 = new RegExp("^(?:/(?:[^~/]|~[01])*)*$", "u");
function validate20(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
  let vErrors = null;
  let errors = 0;
  const evaluated0 = validate20.evaluated;
  if (evaluated0.dynamicProps) {
    evaluated0.props = void 0;
  }
  if (evaluated0.dynamicItems) {
    evaluated0.items = void 0;
  }
  if (errors === 0) {
    if (data && typeof data == "object" && !Array.isArray(data)) {
      let missing0;
      if (data.workflow_version === void 0 && (missing0 = "workflow_version") || data.id === void 0 && (missing0 = "id") || data.name === void 0 && (missing0 = "name") || data.nodes === void 0 && (missing0 = "nodes") || data.edges === void 0 && (missing0 = "edges")) {
        validate20.errors = [{ instancePath, schemaPath: "#/required", keyword: "required", params: { missingProperty: missing0 }, message: "must have required property '" + missing0 + "'" }];
        return false;
      } else {
        const _errs1 = errors;
        for (const key0 in data) {
          if (!(key0 === "workflow_version" || key0 === "id" || key0 === "name" || key0 === "nodes" || key0 === "edges")) {
            validate20.errors = [{ instancePath, schemaPath: "#/additionalProperties", keyword: "additionalProperties", params: { additionalProperty: key0 }, message: "must NOT have additional properties" }];
            return false;
            break;
          }
        }
        if (_errs1 === errors) {
          if (data.workflow_version !== void 0) {
            const _errs2 = errors;
            if ("1.0" !== data.workflow_version) {
              validate20.errors = [{ instancePath: instancePath + "/workflow_version", schemaPath: "#/properties/workflow_version/const", keyword: "const", params: { allowedValue: "1.0" }, message: "must be equal to constant" }];
              return false;
            }
            var valid0 = _errs2 === errors;
          } else {
            var valid0 = true;
          }
          if (valid0) {
            if (data.id !== void 0) {
              let data1 = data.id;
              const _errs3 = errors;
              if (errors === _errs3) {
                if (typeof data1 === "string") {
                  if (func1(data1) > 64) {
                    validate20.errors = [{ instancePath: instancePath + "/id", schemaPath: "#/properties/id/maxLength", keyword: "maxLength", params: { limit: 64 }, message: "must NOT have more than 64 characters" }];
                    return false;
                  } else {
                    if (func1(data1) < 1) {
                      validate20.errors = [{ instancePath: instancePath + "/id", schemaPath: "#/properties/id/minLength", keyword: "minLength", params: { limit: 1 }, message: "must NOT have fewer than 1 characters" }];
                      return false;
                    } else {
                      if (!pattern4.test(data1)) {
                        validate20.errors = [{ instancePath: instancePath + "/id", schemaPath: "#/properties/id/pattern", keyword: "pattern", params: { pattern: "^[A-Za-z][A-Za-z0-9_-]*$" }, message: 'must match pattern "^[A-Za-z][A-Za-z0-9_-]*$"' }];
                        return false;
                      }
                    }
                  }
                } else {
                  validate20.errors = [{ instancePath: instancePath + "/id", schemaPath: "#/properties/id/type", keyword: "type", params: { type: "string" }, message: "must be string" }];
                  return false;
                }
              }
              var valid0 = _errs3 === errors;
            } else {
              var valid0 = true;
            }
            if (valid0) {
              if (data.name !== void 0) {
                let data2 = data.name;
                const _errs5 = errors;
                if (errors === _errs5) {
                  if (typeof data2 === "string") {
                    if (func1(data2) > 256) {
                      validate20.errors = [{ instancePath: instancePath + "/name", schemaPath: "#/properties/name/maxLength", keyword: "maxLength", params: { limit: 256 }, message: "must NOT have more than 256 characters" }];
                      return false;
                    } else {
                      if (func1(data2) < 1) {
                        validate20.errors = [{ instancePath: instancePath + "/name", schemaPath: "#/properties/name/minLength", keyword: "minLength", params: { limit: 1 }, message: "must NOT have fewer than 1 characters" }];
                        return false;
                      }
                    }
                  } else {
                    validate20.errors = [{ instancePath: instancePath + "/name", schemaPath: "#/properties/name/type", keyword: "type", params: { type: "string" }, message: "must be string" }];
                    return false;
                  }
                }
                var valid0 = _errs5 === errors;
              } else {
                var valid0 = true;
              }
              if (valid0) {
                if (data.nodes !== void 0) {
                  let data3 = data.nodes;
                  const _errs7 = errors;
                  if (errors === _errs7) {
                    if (Array.isArray(data3)) {
                      if (data3.length > 100) {
                        validate20.errors = [{ instancePath: instancePath + "/nodes", schemaPath: "#/properties/nodes/maxItems", keyword: "maxItems", params: { limit: 100 }, message: "must NOT have more than 100 items" }];
                        return false;
                      } else {
                        if (data3.length < 1) {
                          validate20.errors = [{ instancePath: instancePath + "/nodes", schemaPath: "#/properties/nodes/minItems", keyword: "minItems", params: { limit: 1 }, message: "must NOT have fewer than 1 items" }];
                          return false;
                        } else {
                          var valid1 = true;
                          const len0 = data3.length;
                          for (let i0 = 0; i0 < len0; i0++) {
                            let data4 = data3[i0];
                            const _errs9 = errors;
                            if (errors === _errs9) {
                              if (data4 && typeof data4 == "object" && !Array.isArray(data4)) {
                                let missing1;
                                if (data4.id === void 0 && (missing1 = "id") || data4.action_ref === void 0 && (missing1 = "action_ref") || data4.input_bindings === void 0 && (missing1 = "input_bindings") || data4.requires_confirmation === void 0 && (missing1 = "requires_confirmation")) {
                                  validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0, schemaPath: "#/properties/nodes/items/required", keyword: "required", params: { missingProperty: missing1 }, message: "must have required property '" + missing1 + "'" }];
                                  return false;
                                } else {
                                  const _errs11 = errors;
                                  for (const key1 in data4) {
                                    if (!(key1 === "id" || key1 === "action_ref" || key1 === "input_bindings" || key1 === "requires_confirmation")) {
                                      validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0, schemaPath: "#/properties/nodes/items/additionalProperties", keyword: "additionalProperties", params: { additionalProperty: key1 }, message: "must NOT have additional properties" }];
                                      return false;
                                      break;
                                    }
                                  }
                                  if (_errs11 === errors) {
                                    if (data4.id !== void 0) {
                                      let data5 = data4.id;
                                      const _errs12 = errors;
                                      if (errors === _errs12) {
                                        if (typeof data5 === "string") {
                                          if (func1(data5) > 64) {
                                            validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0 + "/id", schemaPath: "#/properties/nodes/items/properties/id/maxLength", keyword: "maxLength", params: { limit: 64 }, message: "must NOT have more than 64 characters" }];
                                            return false;
                                          } else {
                                            if (func1(data5) < 1) {
                                              validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0 + "/id", schemaPath: "#/properties/nodes/items/properties/id/minLength", keyword: "minLength", params: { limit: 1 }, message: "must NOT have fewer than 1 characters" }];
                                              return false;
                                            } else {
                                              if (!pattern4.test(data5)) {
                                                validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0 + "/id", schemaPath: "#/properties/nodes/items/properties/id/pattern", keyword: "pattern", params: { pattern: "^[A-Za-z][A-Za-z0-9_-]*$" }, message: 'must match pattern "^[A-Za-z][A-Za-z0-9_-]*$"' }];
                                                return false;
                                              }
                                            }
                                          }
                                        } else {
                                          validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0 + "/id", schemaPath: "#/properties/nodes/items/properties/id/type", keyword: "type", params: { type: "string" }, message: "must be string" }];
                                          return false;
                                        }
                                      }
                                      var valid2 = _errs12 === errors;
                                    } else {
                                      var valid2 = true;
                                    }
                                    if (valid2) {
                                      if (data4.action_ref !== void 0) {
                                        let data6 = data4.action_ref;
                                        const _errs14 = errors;
                                        if (errors === _errs14) {
                                          if (typeof data6 === "string") {
                                            if (func1(data6) > 129) {
                                              validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0 + "/action_ref", schemaPath: "#/properties/nodes/items/properties/action_ref/maxLength", keyword: "maxLength", params: { limit: 129 }, message: "must NOT have more than 129 characters" }];
                                              return false;
                                            } else {
                                              if (func1(data6) < 1) {
                                                validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0 + "/action_ref", schemaPath: "#/properties/nodes/items/properties/action_ref/minLength", keyword: "minLength", params: { limit: 1 }, message: "must NOT have fewer than 1 characters" }];
                                                return false;
                                              } else {
                                                if (!pattern6.test(data6)) {
                                                  validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0 + "/action_ref", schemaPath: "#/properties/nodes/items/properties/action_ref/pattern", keyword: "pattern", params: { pattern: "^[a-z0-9]+(?:-[a-z0-9]+)*\\.[a-z][a-z0-9_-]*$" }, message: 'must match pattern "^[a-z0-9]+(?:-[a-z0-9]+)*\\.[a-z][a-z0-9_-]*$"' }];
                                                  return false;
                                                }
                                              }
                                            }
                                          } else {
                                            validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0 + "/action_ref", schemaPath: "#/properties/nodes/items/properties/action_ref/type", keyword: "type", params: { type: "string" }, message: "must be string" }];
                                            return false;
                                          }
                                        }
                                        var valid2 = _errs14 === errors;
                                      } else {
                                        var valid2 = true;
                                      }
                                      if (valid2) {
                                        if (data4.input_bindings !== void 0) {
                                          let data7 = data4.input_bindings;
                                          const _errs16 = errors;
                                          if (errors === _errs16) {
                                            if (data7 && typeof data7 == "object" && !Array.isArray(data7)) {
                                              if (Object.keys(data7).length > 100) {
                                                validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0 + "/input_bindings", schemaPath: "#/properties/nodes/items/properties/input_bindings/maxProperties", keyword: "maxProperties", params: { limit: 100 }, message: "must NOT have more than 100 properties" }];
                                                return false;
                                              } else {
                                                for (const key2 in data7) {
                                                  const _errs18 = errors;
                                                  if (errors === _errs18) {
                                                    if (typeof key2 === "string") {
                                                      if (func1(key2) > 1024) {
                                                        const err0 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings", schemaPath: "#/properties/nodes/items/properties/input_bindings/propertyNames/maxLength", keyword: "maxLength", params: { limit: 1024 }, message: "must NOT have more than 1024 characters", propertyName: key2 };
                                                        if (vErrors === null) {
                                                          vErrors = [err0];
                                                        } else {
                                                          vErrors.push(err0);
                                                        }
                                                        errors++;
                                                      } else {
                                                        if (!pattern7.test(key2)) {
                                                          const err1 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings", schemaPath: "#/properties/nodes/items/properties/input_bindings/propertyNames/pattern", keyword: "pattern", params: { pattern: "^(?:/(?:[^~/]|~[01])*)*$" }, message: 'must match pattern "^(?:/(?:[^~/]|~[01])*)*$"', propertyName: key2 };
                                                          if (vErrors === null) {
                                                            vErrors = [err1];
                                                          } else {
                                                            vErrors.push(err1);
                                                          }
                                                          errors++;
                                                        }
                                                      }
                                                    } else {
                                                      const err2 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings", schemaPath: "#/properties/nodes/items/properties/input_bindings/propertyNames/type", keyword: "type", params: { type: "string" }, message: "must be string", propertyName: key2 };
                                                      if (vErrors === null) {
                                                        vErrors = [err2];
                                                      } else {
                                                        vErrors.push(err2);
                                                      }
                                                      errors++;
                                                    }
                                                  }
                                                  var valid3 = _errs18 === errors;
                                                  if (!valid3) {
                                                    const err3 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings", schemaPath: "#/properties/nodes/items/properties/input_bindings/propertyNames", keyword: "propertyNames", params: { propertyName: key2 }, message: "property name must be valid" };
                                                    if (vErrors === null) {
                                                      vErrors = [err3];
                                                    } else {
                                                      vErrors.push(err3);
                                                    }
                                                    errors++;
                                                    validate20.errors = vErrors;
                                                    return false;
                                                    break;
                                                  }
                                                }
                                                if (valid3) {
                                                  for (const key3 in data7) {
                                                    let data8 = data7[key3];
                                                    const _errs21 = errors;
                                                    const _errs22 = errors;
                                                    let valid5 = false;
                                                    let passing0 = null;
                                                    const _errs23 = errors;
                                                    if (errors === _errs23) {
                                                      if (data8 && typeof data8 == "object" && !Array.isArray(data8)) {
                                                        let missing2;
                                                        if (data8.value === void 0 && (missing2 = "value")) {
                                                          const err4 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1"), schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/0/required", keyword: "required", params: { missingProperty: missing2 }, message: "must have required property '" + missing2 + "'" };
                                                          if (vErrors === null) {
                                                            vErrors = [err4];
                                                          } else {
                                                            vErrors.push(err4);
                                                          }
                                                          errors++;
                                                        } else {
                                                          for (const key4 in data8) {
                                                            if (!(key4 === "value")) {
                                                              const err5 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1"), schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/0/additionalProperties", keyword: "additionalProperties", params: { additionalProperty: key4 }, message: "must NOT have additional properties" };
                                                              if (vErrors === null) {
                                                                vErrors = [err5];
                                                              } else {
                                                                vErrors.push(err5);
                                                              }
                                                              errors++;
                                                              break;
                                                            }
                                                          }
                                                        }
                                                      } else {
                                                        const err6 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1"), schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/0/type", keyword: "type", params: { type: "object" }, message: "must be object" };
                                                        if (vErrors === null) {
                                                          vErrors = [err6];
                                                        } else {
                                                          vErrors.push(err6);
                                                        }
                                                        errors++;
                                                      }
                                                    }
                                                    var _valid0 = _errs23 === errors;
                                                    if (_valid0) {
                                                      valid5 = true;
                                                      passing0 = 0;
                                                      var props0 = true;
                                                    }
                                                    const _errs26 = errors;
                                                    if (errors === _errs26) {
                                                      if (data8 && typeof data8 == "object" && !Array.isArray(data8)) {
                                                        let missing3;
                                                        if (data8.from_node === void 0 && (missing3 = "from_node") || data8.source_pointer === void 0 && (missing3 = "source_pointer")) {
                                                          const err7 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1"), schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/1/required", keyword: "required", params: { missingProperty: missing3 }, message: "must have required property '" + missing3 + "'" };
                                                          if (vErrors === null) {
                                                            vErrors = [err7];
                                                          } else {
                                                            vErrors.push(err7);
                                                          }
                                                          errors++;
                                                        } else {
                                                          const _errs28 = errors;
                                                          for (const key5 in data8) {
                                                            if (!(key5 === "from_node" || key5 === "source_pointer")) {
                                                              const err8 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1"), schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/1/additionalProperties", keyword: "additionalProperties", params: { additionalProperty: key5 }, message: "must NOT have additional properties" };
                                                              if (vErrors === null) {
                                                                vErrors = [err8];
                                                              } else {
                                                                vErrors.push(err8);
                                                              }
                                                              errors++;
                                                              break;
                                                            }
                                                          }
                                                          if (_errs28 === errors) {
                                                            if (data8.from_node !== void 0) {
                                                              let data9 = data8.from_node;
                                                              const _errs29 = errors;
                                                              if (errors === _errs29) {
                                                                if (typeof data9 === "string") {
                                                                  if (func1(data9) > 64) {
                                                                    const err9 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1") + "/from_node", schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/1/properties/from_node/maxLength", keyword: "maxLength", params: { limit: 64 }, message: "must NOT have more than 64 characters" };
                                                                    if (vErrors === null) {
                                                                      vErrors = [err9];
                                                                    } else {
                                                                      vErrors.push(err9);
                                                                    }
                                                                    errors++;
                                                                  } else {
                                                                    if (func1(data9) < 1) {
                                                                      const err10 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1") + "/from_node", schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/1/properties/from_node/minLength", keyword: "minLength", params: { limit: 1 }, message: "must NOT have fewer than 1 characters" };
                                                                      if (vErrors === null) {
                                                                        vErrors = [err10];
                                                                      } else {
                                                                        vErrors.push(err10);
                                                                      }
                                                                      errors++;
                                                                    } else {
                                                                      if (!pattern4.test(data9)) {
                                                                        const err11 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1") + "/from_node", schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/1/properties/from_node/pattern", keyword: "pattern", params: { pattern: "^[A-Za-z][A-Za-z0-9_-]*$" }, message: 'must match pattern "^[A-Za-z][A-Za-z0-9_-]*$"' };
                                                                        if (vErrors === null) {
                                                                          vErrors = [err11];
                                                                        } else {
                                                                          vErrors.push(err11);
                                                                        }
                                                                        errors++;
                                                                      }
                                                                    }
                                                                  }
                                                                } else {
                                                                  const err12 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1") + "/from_node", schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/1/properties/from_node/type", keyword: "type", params: { type: "string" }, message: "must be string" };
                                                                  if (vErrors === null) {
                                                                    vErrors = [err12];
                                                                  } else {
                                                                    vErrors.push(err12);
                                                                  }
                                                                  errors++;
                                                                }
                                                              }
                                                              var valid6 = _errs29 === errors;
                                                            } else {
                                                              var valid6 = true;
                                                            }
                                                            if (valid6) {
                                                              if (data8.source_pointer !== void 0) {
                                                                let data10 = data8.source_pointer;
                                                                const _errs31 = errors;
                                                                if (errors === _errs31) {
                                                                  if (typeof data10 === "string") {
                                                                    if (func1(data10) > 1024) {
                                                                      const err13 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1") + "/source_pointer", schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/1/properties/source_pointer/maxLength", keyword: "maxLength", params: { limit: 1024 }, message: "must NOT have more than 1024 characters" };
                                                                      if (vErrors === null) {
                                                                        vErrors = [err13];
                                                                      } else {
                                                                        vErrors.push(err13);
                                                                      }
                                                                      errors++;
                                                                    } else {
                                                                      if (!pattern7.test(data10)) {
                                                                        const err14 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1") + "/source_pointer", schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/1/properties/source_pointer/pattern", keyword: "pattern", params: { pattern: "^(?:/(?:[^~/]|~[01])*)*$" }, message: 'must match pattern "^(?:/(?:[^~/]|~[01])*)*$"' };
                                                                        if (vErrors === null) {
                                                                          vErrors = [err14];
                                                                        } else {
                                                                          vErrors.push(err14);
                                                                        }
                                                                        errors++;
                                                                      }
                                                                    }
                                                                  } else {
                                                                    const err15 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1") + "/source_pointer", schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/1/properties/source_pointer/type", keyword: "type", params: { type: "string" }, message: "must be string" };
                                                                    if (vErrors === null) {
                                                                      vErrors = [err15];
                                                                    } else {
                                                                      vErrors.push(err15);
                                                                    }
                                                                    errors++;
                                                                  }
                                                                }
                                                                var valid6 = _errs31 === errors;
                                                              } else {
                                                                var valid6 = true;
                                                              }
                                                            }
                                                          }
                                                        }
                                                      } else {
                                                        const err16 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1"), schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf/1/type", keyword: "type", params: { type: "object" }, message: "must be object" };
                                                        if (vErrors === null) {
                                                          vErrors = [err16];
                                                        } else {
                                                          vErrors.push(err16);
                                                        }
                                                        errors++;
                                                      }
                                                    }
                                                    var _valid0 = _errs26 === errors;
                                                    if (_valid0 && valid5) {
                                                      valid5 = false;
                                                      passing0 = [passing0, 1];
                                                    } else {
                                                      if (_valid0) {
                                                        valid5 = true;
                                                        passing0 = 1;
                                                        if (props0 !== true) {
                                                          props0 = true;
                                                        }
                                                      }
                                                    }
                                                    if (!valid5) {
                                                      const err17 = { instancePath: instancePath + "/nodes/" + i0 + "/input_bindings/" + key3.replace(/~/g, "~0").replace(/\//g, "~1"), schemaPath: "#/properties/nodes/items/properties/input_bindings/additionalProperties/oneOf", keyword: "oneOf", params: { passingSchemas: passing0 }, message: "must match exactly one schema in oneOf" };
                                                      if (vErrors === null) {
                                                        vErrors = [err17];
                                                      } else {
                                                        vErrors.push(err17);
                                                      }
                                                      errors++;
                                                      validate20.errors = vErrors;
                                                      return false;
                                                    } else {
                                                      errors = _errs22;
                                                      if (vErrors !== null) {
                                                        if (_errs22) {
                                                          vErrors.length = _errs22;
                                                        } else {
                                                          vErrors = null;
                                                        }
                                                      }
                                                    }
                                                    var valid4 = _errs21 === errors;
                                                    if (!valid4) {
                                                      break;
                                                    }
                                                  }
                                                }
                                              }
                                            } else {
                                              validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0 + "/input_bindings", schemaPath: "#/properties/nodes/items/properties/input_bindings/type", keyword: "type", params: { type: "object" }, message: "must be object" }];
                                              return false;
                                            }
                                          }
                                          var valid2 = _errs16 === errors;
                                        } else {
                                          var valid2 = true;
                                        }
                                        if (valid2) {
                                          if (data4.requires_confirmation !== void 0) {
                                            const _errs33 = errors;
                                            if (typeof data4.requires_confirmation !== "boolean") {
                                              validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0 + "/requires_confirmation", schemaPath: "#/properties/nodes/items/properties/requires_confirmation/type", keyword: "type", params: { type: "boolean" }, message: "must be boolean" }];
                                              return false;
                                            }
                                            var valid2 = _errs33 === errors;
                                          } else {
                                            var valid2 = true;
                                          }
                                        }
                                      }
                                    }
                                  }
                                }
                              } else {
                                validate20.errors = [{ instancePath: instancePath + "/nodes/" + i0, schemaPath: "#/properties/nodes/items/type", keyword: "type", params: { type: "object" }, message: "must be object" }];
                                return false;
                              }
                            }
                            var valid1 = _errs9 === errors;
                            if (!valid1) {
                              break;
                            }
                          }
                        }
                      }
                    } else {
                      validate20.errors = [{ instancePath: instancePath + "/nodes", schemaPath: "#/properties/nodes/type", keyword: "type", params: { type: "array" }, message: "must be array" }];
                      return false;
                    }
                  }
                  var valid0 = _errs7 === errors;
                } else {
                  var valid0 = true;
                }
                if (valid0) {
                  if (data.edges !== void 0) {
                    let data12 = data.edges;
                    const _errs35 = errors;
                    if (errors === _errs35) {
                      if (Array.isArray(data12)) {
                        if (data12.length > 1e3) {
                          validate20.errors = [{ instancePath: instancePath + "/edges", schemaPath: "#/properties/edges/maxItems", keyword: "maxItems", params: { limit: 1e3 }, message: "must NOT have more than 1000 items" }];
                          return false;
                        } else {
                          var valid7 = true;
                          const len1 = data12.length;
                          for (let i1 = 0; i1 < len1; i1++) {
                            let data13 = data12[i1];
                            const _errs37 = errors;
                            if (errors === _errs37) {
                              if (data13 && typeof data13 == "object" && !Array.isArray(data13)) {
                                let missing4;
                                if (data13.from === void 0 && (missing4 = "from") || data13.to === void 0 && (missing4 = "to")) {
                                  validate20.errors = [{ instancePath: instancePath + "/edges/" + i1, schemaPath: "#/properties/edges/items/required", keyword: "required", params: { missingProperty: missing4 }, message: "must have required property '" + missing4 + "'" }];
                                  return false;
                                } else {
                                  const _errs39 = errors;
                                  for (const key6 in data13) {
                                    if (!(key6 === "from" || key6 === "to")) {
                                      validate20.errors = [{ instancePath: instancePath + "/edges/" + i1, schemaPath: "#/properties/edges/items/additionalProperties", keyword: "additionalProperties", params: { additionalProperty: key6 }, message: "must NOT have additional properties" }];
                                      return false;
                                      break;
                                    }
                                  }
                                  if (_errs39 === errors) {
                                    if (data13.from !== void 0) {
                                      let data14 = data13.from;
                                      const _errs40 = errors;
                                      if (errors === _errs40) {
                                        if (typeof data14 === "string") {
                                          if (func1(data14) > 64) {
                                            validate20.errors = [{ instancePath: instancePath + "/edges/" + i1 + "/from", schemaPath: "#/properties/edges/items/properties/from/maxLength", keyword: "maxLength", params: { limit: 64 }, message: "must NOT have more than 64 characters" }];
                                            return false;
                                          } else {
                                            if (func1(data14) < 1) {
                                              validate20.errors = [{ instancePath: instancePath + "/edges/" + i1 + "/from", schemaPath: "#/properties/edges/items/properties/from/minLength", keyword: "minLength", params: { limit: 1 }, message: "must NOT have fewer than 1 characters" }];
                                              return false;
                                            } else {
                                              if (!pattern4.test(data14)) {
                                                validate20.errors = [{ instancePath: instancePath + "/edges/" + i1 + "/from", schemaPath: "#/properties/edges/items/properties/from/pattern", keyword: "pattern", params: { pattern: "^[A-Za-z][A-Za-z0-9_-]*$" }, message: 'must match pattern "^[A-Za-z][A-Za-z0-9_-]*$"' }];
                                                return false;
                                              }
                                            }
                                          }
                                        } else {
                                          validate20.errors = [{ instancePath: instancePath + "/edges/" + i1 + "/from", schemaPath: "#/properties/edges/items/properties/from/type", keyword: "type", params: { type: "string" }, message: "must be string" }];
                                          return false;
                                        }
                                      }
                                      var valid8 = _errs40 === errors;
                                    } else {
                                      var valid8 = true;
                                    }
                                    if (valid8) {
                                      if (data13.to !== void 0) {
                                        let data15 = data13.to;
                                        const _errs42 = errors;
                                        if (errors === _errs42) {
                                          if (typeof data15 === "string") {
                                            if (func1(data15) > 64) {
                                              validate20.errors = [{ instancePath: instancePath + "/edges/" + i1 + "/to", schemaPath: "#/properties/edges/items/properties/to/maxLength", keyword: "maxLength", params: { limit: 64 }, message: "must NOT have more than 64 characters" }];
                                              return false;
                                            } else {
                                              if (func1(data15) < 1) {
                                                validate20.errors = [{ instancePath: instancePath + "/edges/" + i1 + "/to", schemaPath: "#/properties/edges/items/properties/to/minLength", keyword: "minLength", params: { limit: 1 }, message: "must NOT have fewer than 1 characters" }];
                                                return false;
                                              } else {
                                                if (!pattern4.test(data15)) {
                                                  validate20.errors = [{ instancePath: instancePath + "/edges/" + i1 + "/to", schemaPath: "#/properties/edges/items/properties/to/pattern", keyword: "pattern", params: { pattern: "^[A-Za-z][A-Za-z0-9_-]*$" }, message: 'must match pattern "^[A-Za-z][A-Za-z0-9_-]*$"' }];
                                                  return false;
                                                }
                                              }
                                            }
                                          } else {
                                            validate20.errors = [{ instancePath: instancePath + "/edges/" + i1 + "/to", schemaPath: "#/properties/edges/items/properties/to/type", keyword: "type", params: { type: "string" }, message: "must be string" }];
                                            return false;
                                          }
                                        }
                                        var valid8 = _errs42 === errors;
                                      } else {
                                        var valid8 = true;
                                      }
                                    }
                                  }
                                }
                              } else {
                                validate20.errors = [{ instancePath: instancePath + "/edges/" + i1, schemaPath: "#/properties/edges/items/type", keyword: "type", params: { type: "object" }, message: "must be object" }];
                                return false;
                              }
                            }
                            var valid7 = _errs37 === errors;
                            if (!valid7) {
                              break;
                            }
                          }
                        }
                      } else {
                        validate20.errors = [{ instancePath: instancePath + "/edges", schemaPath: "#/properties/edges/type", keyword: "type", params: { type: "array" }, message: "must be array" }];
                        return false;
                      }
                    }
                    var valid0 = _errs35 === errors;
                  } else {
                    var valid0 = true;
                  }
                }
              }
            }
          }
        }
      }
    } else {
      validate20.errors = [{ instancePath, schemaPath: "#/type", keyword: "type", params: { type: "object" }, message: "must be object" }];
      return false;
    }
  }
  validate20.errors = vErrors;
  return errors === 0;
}
validate20.evaluated = { "props": true, "dynamicProps": false, "dynamicItems": false };
export {
  workflow
};
