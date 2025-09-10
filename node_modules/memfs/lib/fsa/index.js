"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fsa = exports.coreToFsa = void 0;
const tslib_1 = require("tslib");
const CoreFileSystemDirectoryHandle_1 = require("./CoreFileSystemDirectoryHandle");
const Superblock_1 = require("../core/Superblock");
tslib_1.__exportStar(require("./types"), exports);
tslib_1.__exportStar(require("./CoreFileSystemHandle"), exports);
tslib_1.__exportStar(require("./CoreFileSystemDirectoryHandle"), exports);
tslib_1.__exportStar(require("./CoreFileSystemFileHandle"), exports);
tslib_1.__exportStar(require("./CoreFileSystemSyncAccessHandle"), exports);
tslib_1.__exportStar(require("./CoreFileSystemWritableFileStream"), exports);
tslib_1.__exportStar(require("./CorePermissionStatus"), exports);
/**
 * Creates a File System Access API implementation on top of a Superblock.
 */
const coreToFsa = (core, dirPath = '/', ctx) => {
    return new CoreFileSystemDirectoryHandle_1.CoreFileSystemDirectoryHandle(core, dirPath, ctx);
};
exports.coreToFsa = coreToFsa;
/**
 * Create a new instance of an in-memory File System Access API
 * implementation rooted at the root directory of the filesystem.
 *
 * @param ctx Optional context for the File System Access API.
 * @returns A File System Access API implementation `dir` rooted at
 *     the root directory of the filesystem, as well as the `core`
 *     file system itself.
 */
const fsa = (ctx) => {
    const core = new Superblock_1.Superblock();
    const dir = new CoreFileSystemDirectoryHandle_1.CoreFileSystemDirectoryHandle(core, '/', ctx);
    return { dir, core };
};
exports.fsa = fsa;
//# sourceMappingURL=index.js.map