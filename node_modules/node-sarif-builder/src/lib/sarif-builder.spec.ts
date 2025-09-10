import * as os from 'os';
import * as path from 'path';

import test from 'ava';
import * as fs from 'fs-extra';
import { Log } from 'sarif';

import { SarifBuilder } from './sarif-builder';
import { SarifResultBuilder } from './sarif-result-builder';
import { SarifRuleBuilder } from './sarif-rule-builder';
import { SarifRunBuilder } from './sarif-run-builder';

test('Create SarifBuilder', (t) => {
  const sarifBuilder = new SarifBuilder();
  t.assert(sarifBuilder !== null, 'SarifBuilder has been created');
});

test('Create SarifBuilder with args', (t) => {
  const sarifBuilder = new SarifBuilder({
    $schema: 'http://json.schemastore.org/sarif-2.1.0-rtm.3',
  });
  t.is(
    sarifBuilder.log.$schema,
    'http://json.schemastore.org/sarif-2.1.0-rtm.3',
  );
});

test('Create SarifRunBuilder', (t) => {
  const sarifBuilder = new SarifRunBuilder();
  t.assert(sarifBuilder != null, 'SarifRunBuilder has been created');
});

test('Create SarifRunBuilder and use initSimple', (t) => {
  const sarifRunBuilder = createInitSarifRunBuilder();
  t.assert(sarifRunBuilder != null, 'SarifRunBuilder has been created');
  t.is(sarifRunBuilder.run.tool.driver.name, 'MegaLinter');
});

test('Create SarifResultBuilder', (t) => {
  const sarifResultBuilder = new SarifResultBuilder();
  t.assert(sarifResultBuilder != null, 'SarifResultBuilder has been created');
});

test('Create SarifResultBuilder and set message', (t) => {
  const sarifResultBuilder = new SarifResultBuilder();
  sarifResultBuilder.setMessageText('MegaLinter message');
  t.assert(sarifResultBuilder != null, 'SarifResultBuilder has been created');
  t.is(sarifResultBuilder.result.message.text, 'MegaLinter message');
});

test('Create SarifResultBuilder and use initSimple', (t) => {
  const sarifResultBuilder = createInitSarifResultBuilder();
  t.assert(sarifResultBuilder != null, 'SarifResultBuilder has been created');
  t.is(
    sarifResultBuilder.result.message.text,
    'An assignment operator (=) was used in a conditional test. This is usually a typo, and the comparison operator (==) was intended.',
  );
  t.is(sarifResultBuilder.result.ruleId, 'AssignmentInConditional');
  t.is(
    sarifResultBuilder.result.locations[0].physicalLocation.artifactLocation
      .uri,
    'src/urf/wesh.js',
  );
  t.is(
    sarifResultBuilder.result.locations[0].physicalLocation.region.startLine,
    8,
  );
  t.is(
    sarifResultBuilder.result.locations[0].physicalLocation.region.startColumn,
    1,
  );
  t.is(
    sarifResultBuilder.result.locations[0].physicalLocation.region.endLine,
    8,
  );
  t.is(
    sarifResultBuilder.result.locations[0].physicalLocation.region.endColumn,
    1,
  );
});

test('Create SarifResultBuilder and generate file', (t) => {
  const sarifBuilder = new SarifBuilder();
  const sarifRunBuilder = createInitSarifRunBuilder();
  sarifRunBuilder.addRule(createInitSarifRuleBuilder());
  sarifRunBuilder.addRule(createInitSarifRuleBuilder2());
  sarifRunBuilder.addResult(createInitSarifResultBuilder());
  sarifRunBuilder.addResult(createInitSarifResultBuilder2());
  sarifBuilder.addRun(sarifRunBuilder);
  const outputFile = path.join(
    os.tmpdir(),
    'testSarifBuilder-' + Math.random() + '.sarif',
  );
  sarifBuilder.generateSarifFileSync(outputFile);
  t.assert(fs.existsSync(outputFile), 'Output SARIF file not found');
  const outputSarifObj: Log = JSON.parse(fs.readFileSync(outputFile, 'utf8'));
  t.assert(
    outputSarifObj?.runs?.length > 0,
    'No runs found in generated SARIF log',
  );
  t.assert(
    outputSarifObj?.runs[0].tool?.driver?.rules?.length > 1,
    'No rules found in generated SARIF log',
  );
  t.assert(
    outputSarifObj?.runs[0].artifacts.length > 0,
    'No artifacts found in generated SARIF log',
  );
  t.assert(
    outputSarifObj?.runs[0].results?.length > 1,
    'No results found in generated SARIF log',
  );
  t.assert(
    outputSarifObj?.runs[0].results[0].ruleIndex !== null,
    'Result rule index should be set',
  );
  t.assert(
    outputSarifObj?.runs[0].results[0]?.locations[0]?.physicalLocation
      ?.artifactLocation?.index !== null,
    'Result artifact index should be set',
  );
});

test('Create SarifResultBuilder with error', (t) => {
  let error = false;
  try {
    createInitSarifWrongResultBuilder();
  } catch (e) {
    error = true;
    console.log('Error: ' + e.message);
  }
  t.assert(error === true, 'Error should have been triggered');
});

function createInitSarifRunBuilder() {
  const sarifRunBuilder = new SarifRunBuilder();
  sarifRunBuilder.initSimple({
    toolDriverName: 'MegaLinter',
    toolDriverVersion: '5.5.0',
  });
  return sarifRunBuilder;
}

function createInitSarifResultBuilder() {
  const sarifResultBuilder = new SarifResultBuilder();
  sarifResultBuilder.initSimple({
    level: 'warning',
    messageText:
      'An assignment operator (=) was used in a conditional test. This is usually a typo, and the comparison operator (==) was intended.',
    ruleId: 'AssignmentInConditional',
    fileUri: 'src/urf/wesh.js',
    startLine: 8,
  });
  return sarifResultBuilder;
}

function createInitSarifResultBuilder2() {
  const sarifResultBuilder = new SarifResultBuilder();
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
  const sarifResultBuilder = new SarifResultBuilder();
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
  const sarifRuleBuilder = new SarifRuleBuilder();
  sarifRuleBuilder.initSimple({
    ruleId: 'AssignmentInConditional',
    shortDescriptionText:
      'This is wrong, that should not happenAn assignment operator (=) was used in a conditional test. This is usually a typo, and the comparison operator (==) was intended.',
    fullDescriptionText:
      'Change something in your code and this rule will not be triggered !',
    helpUri:
      'https://codenarc.org/codenarc-rules-basic.html#AssignmentInConditional',
  });
  return sarifRuleBuilder;
}

function createInitSarifRuleBuilder2() {
  const sarifRuleBuilder = new SarifRuleBuilder();
  sarifRuleBuilder.initSimple({
    ruleId: 'NoAny',
    shortDescriptionText: 'Nooo no no, any are not good !',
    helpUri: 'https://codenarc.org/codenarc-rules-basic.html#NoAny',
  });
  return sarifRuleBuilder;
}
