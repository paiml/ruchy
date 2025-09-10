"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.flattenJSON = void 0;
const buffer_1 = require("../internal/buffer");
const pathModule = require("path");
const { join } = pathModule.posix ? pathModule.posix : pathModule;
const flattenJSON = (nestedJSON) => {
    const flatJSON = {};
    function flatten(pathPrefix, node) {
        for (const path in node) {
            const contentOrNode = node[path];
            // TODO: Can we avoid using `join` here? Just concatenate?
            const joinedPath = join(pathPrefix, path);
            if (typeof contentOrNode === 'string' || contentOrNode instanceof buffer_1.Buffer) {
                flatJSON[joinedPath] = contentOrNode;
            }
            else if (typeof contentOrNode === 'object' && contentOrNode !== null && Object.keys(contentOrNode).length > 0) {
                // empty directories need an explicit entry and therefore get handled in `else`, non-empty ones are implicitly considered
                flatten(joinedPath, contentOrNode);
            }
            else {
                // without this branch null, empty-object or non-object entries would not be handled in the same way
                // by both fromJSON() and fromNestedJSON()
                flatJSON[joinedPath] = null;
            }
        }
    }
    flatten('', nestedJSON);
    return flatJSON;
};
exports.flattenJSON = flattenJSON;
//# sourceMappingURL=json.js.map