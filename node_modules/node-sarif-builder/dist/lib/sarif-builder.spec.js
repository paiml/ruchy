"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const os = require("os");
const path = require("path");
const ava_1 = require("ava");
const fs = require("fs-extra");
const sarif_builder_1 = require("./sarif-builder");
const sarif_result_builder_1 = require("./sarif-result-builder");
const sarif_rule_builder_1 = require("./sarif-rule-builder");
const sarif_run_builder_1 = require("./sarif-run-builder");
(0, ava_1.default)('Create SarifBuilder', (t) => {
    const sarifBuilder = new sarif_builder_1.SarifBuilder();
    t.assert(sarifBuilder !== null, 'SarifBuilder has been created');
});
(0, ava_1.default)('Create SarifBuilder with args', (t) => {
    const sarifBuilder = new sarif_builder_1.SarifBuilder({
        $schema: 'http://json.schemastore.org/sarif-2.1.0-rtm.3',
    });
    t.is(sarifBuilder.log.$schema, 'http://json.schemastore.org/sarif-2.1.0-rtm.3');
});
(0, ava_1.default)('Create SarifRunBuilder', (t) => {
    const sarifBuilder = new sarif_run_builder_1.SarifRunBuilder();
    t.assert(sarifBuilder != null, 'SarifRunBuilder has been created');
});
(0, ava_1.default)('Create SarifRunBuilder and use initSimple', (t) => {
    const sarifRunBuilder = createInitSarifRunBuilder();
    t.assert(sarifRunBuilder != null, 'SarifRunBuilder has been created');
    t.is(sarifRunBuilder.run.tool.driver.name, 'MegaLinter');
});
(0, ava_1.default)('Create SarifResultBuilder', (t) => {
    const sarifResultBuilder = new sarif_result_builder_1.SarifResultBuilder();
    t.assert(sarifResultBuilder != null, 'SarifResultBuilder has been created');
});
(0, ava_1.default)('Create SarifResultBuilder and set message', (t) => {
    const sarifResultBuilder = new sarif_result_builder_1.SarifResultBuilder();
    sarifResultBuilder.setMessageText('MegaLinter message');
    t.assert(sarifResultBuilder != null, 'SarifResultBuilder has been created');
    t.is(sarifResultBuilder.result.message.text, 'MegaLinter message');
});
(0, ava_1.default)('Create SarifResultBuilder and use initSimple', (t) => {
    const sarifResultBuilder = createInitSarifResultBuilder();
    t.assert(sarifResultBuilder != null, 'SarifResultBuilder has been created');
    t.is(sarifResultBuilder.result.message.text, 'An assignment operator (=) was used in a conditional test. This is usually a typo, and the comparison operator (==) was intended.');
    t.is(sarifResultBuilder.result.ruleId, 'AssignmentInConditional');
    t.is(sarifResultBuilder.result.locations[0].physicalLocation.artifactLocation
        .uri, 'src/urf/wesh.js');
    t.is(sarifResultBuilder.result.locations[0].physicalLocation.region.startLine, 8);
    t.is(sarifResultBuilder.result.locations[0].physicalLocation.region.startColumn, 1);
    t.is(sarifResultBuilder.result.locations[0].physicalLocation.region.endLine, 8);
    t.is(sarifResultBuilder.result.locations[0].physicalLocation.region.endColumn, 1);
});
(0, ava_1.default)('Create SarifResultBuilder and generate file', (t) => {
    var _a, _b, _c, _d, _e, _f, _g, _h, _j;
    const sarifBuilder = new sarif_builder_1.SarifBuilder();
    const sarifRunBuilder = createInitSarifRunBuilder();
    sarifRunBuilder.addRule(createInitSarifRuleBuilder());
    sarifRunBuilder.addRule(createInitSarifRuleBuilder2());
    sarifRunBuilder.addResult(createInitSarifResultBuilder());
    sarifRunBuilder.addResult(createInitSarifResultBuilder2());
    sarifBuilder.addRun(sarifRunBuilder);
    const outputFile = path.join(os.tmpdir(), 'testSarifBuilder-' + Math.random() + '.sarif');
    sarifBuilder.generateSarifFileSync(outputFile);
    t.assert(fs.existsSync(outputFile), 'Output SARIF file not found');
    const outputSarifObj = JSON.parse(fs.readFileSync(outputFile, 'utf8'));
    t.assert(((_a = outputSarifObj === null || outputSarifObj === void 0 ? void 0 : outputSarifObj.runs) === null || _a === void 0 ? void 0 : _a.length) > 0, 'No runs found in generated SARIF log');
    t.assert(((_d = (_c = (_b = outputSarifObj === null || outputSarifObj === void 0 ? void 0 : outputSarifObj.runs[0].tool) === null || _b === void 0 ? void 0 : _b.driver) === null || _c === void 0 ? void 0 : _c.rules) === null || _d === void 0 ? void 0 : _d.length) > 1, 'No rules found in generated SARIF log');
    t.assert((outputSarifObj === null || outputSarifObj === void 0 ? void 0 : outputSarifObj.runs[0].artifacts.length) > 0, 'No artifacts found in generated SARIF log');
    t.assert(((_e = outputSarifObj === null || outputSarifObj === void 0 ? void 0 : outputSarifObj.runs[0].results) === null || _e === void 0 ? void 0 : _e.length) > 1, 'No results found in generated SARIF log');
    t.assert((outputSarifObj === null || outputSarifObj === void 0 ? void 0 : outputSarifObj.runs[0].results[0].ruleIndex) !== null, 'Result rule index should be set');
    t.assert(((_j = (_h = (_g = (_f = outputSarifObj === null || outputSarifObj === void 0 ? void 0 : outputSarifObj.runs[0].results[0]) === null || _f === void 0 ? void 0 : _f.locations[0]) === null || _g === void 0 ? void 0 : _g.physicalLocation) === null || _h === void 0 ? void 0 : _h.artifactLocation) === null || _j === void 0 ? void 0 : _j.index) !== null, 'Result artifact index should be set');
});
(0, ava_1.default)('Create SarifResultBuilder with error', (t) => {
    let error = false;
    try {
        createInitSarifWrongResultBuilder();
    }
    catch (e) {
        error = true;
        console.log('Error: ' + e.message);
    }
    t.assert(error === true, 'Error should have been triggered');
});
function createInitSarifRunBuilder() {
    const sarifRunBuilder = new sarif_run_builder_1.SarifRunBuilder();
    sarifRunBuilder.initSimple({
        toolDriverName: 'MegaLinter',
        toolDriverVersion: '5.5.0',
    });
    return sarifRunBuilder;
}
function createInitSarifResultBuilder() {
    const sarifResultBuilder = new sarif_result_builder_1.SarifResultBuilder();
    sarifResultBuilder.initSimple({
        level: 'warning',
        messageText: 'An assignment operator (=) was used in a conditional test. This is usually a typo, and the comparison operator (==) was intended.',
        ruleId: 'AssignmentInConditional',
        fileUri: 'src/urf/wesh.js',
        startLine: 8,
    });
    return sarifResultBuilder;
}
function createInitSarifResultBuilder2() {
    const sarifResultBuilder = new sarif_result_builder_1.SarifResultBuilder();
    sarifResultBuilder.initSimple({
        level: 'warning',
        messageText: 'Nooo no any !',
        ruleId: 'NoAny',
        fileUri: 'src/urf/wesh.js',
        startLine: 8,
    });
    return sarifResultBuilder;
}
function createInitSarifWrongResultBuilder() {
    const sarifResultBuilder = new sarif_result_builder_1.SarifResultBuilder();
    sarifResultBuilder.initSimple({
        level: 'warning',
        messageText: 'some code used = , you may should have used ==',
        ruleId: 'AssignmentInConditional',
        fileUri: 'src/urf/wesh.js',
        startLine: 0,
    });
    return sarifResultBuilder;
}
function createInitSarifRuleBuilder() {
    const sarifRuleBuilder = new sarif_rule_builder_1.SarifRuleBuilder();
    sarifRuleBuilder.initSimple({
        ruleId: 'AssignmentInConditional',
        shortDescriptionText: 'This is wrong, that should not happenAn assignment operator (=) was used in a conditional test. This is usually a typo, and the comparison operator (==) was intended.',
        fullDescriptionText: 'Change something in your code and this rule will not be triggered !',
        helpUri: 'https://codenarc.org/codenarc-rules-basic.html#AssignmentInConditional',
    });
    return sarifRuleBuilder;
}
function createInitSarifRuleBuilder2() {
    const sarifRuleBuilder = new sarif_rule_builder_1.SarifRuleBuilder();
    sarifRuleBuilder.initSimple({
        ruleId: 'NoAny',
        shortDescriptionText: 'Nooo no no, any are not good !',
        helpUri: 'https://codenarc.org/codenarc-rules-basic.html#NoAny',
    });
    return sarifRuleBuilder;
}
