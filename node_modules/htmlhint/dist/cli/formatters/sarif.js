"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const fs_1 = require("fs");
const node_sarif_builder_1 = require("node-sarif-builder");
const path = require("path");
const pkg = require('../../../package.json');
function getRuleMarkdown(ruleId) {
    const possiblePaths = [
        path.join(process.cwd(), 'website', 'src', 'content', 'docs', 'rules', `${ruleId}.mdx`),
        path.join(path.dirname(require.resolve('../../../package.json')), 'website', 'src', 'content', 'docs', 'rules', `${ruleId}.mdx`),
        path.join(process.cwd(), 'src', 'content', 'docs', 'rules', `${ruleId}.mdx`),
    ];
    for (const mdxFilePath of possiblePaths) {
        try {
            if ((0, fs_1.existsSync)(mdxFilePath)) {
                const content = (0, fs_1.readFileSync)(mdxFilePath, 'utf8');
                const frontmatterEnd = content.indexOf('---', 4) + 3;
                if (frontmatterEnd > 3) {
                    const markdown = content.substring(frontmatterEnd).trim();
                    const lines = markdown.split(/\r?\n/);
                    const filteredLines = lines.filter((line) => !line.includes("import { Badge } from '@astrojs/starlight/components';"));
                    let processedMarkdown = filteredLines.join('\n');
                    processedMarkdown = processedMarkdown.replace(/<Badge\s+text="([^"]+)"[^>]*\/>/g, '$1');
                    processedMarkdown = processedMarkdown.replace(/(<\/?[a-zA-Z][^>\s]*[^>]*>|<!DOCTYPE[^>]*>)/g, '`$1`');
                    processedMarkdown = processedMarkdown.replace(/```html\n([\s\S]*?)```/g, (match, code) => `\`\`\`html\n${code}\`\`\``);
                    return processedMarkdown;
                }
            }
        }
        catch (error) {
        }
    }
    return undefined;
}
const sarifFormatter = function (formatter) {
    formatter.on('end', (event) => {
        const arrAllMessages = event.arrAllMessages;
        const sarifBuilder = new node_sarif_builder_1.SarifBuilder();
        const sarifRunBuilder = new node_sarif_builder_1.SarifRunBuilder().initSimple({
            toolDriverName: 'HTMLHint',
            toolDriverVersion: pkg.version,
            url: 'https://htmlhint.com/',
        });
        const addedRuleSet = new Set();
        arrAllMessages.forEach((result) => {
            result.messages.forEach((message) => {
                const rule = message.rule;
                if (addedRuleSet.has(rule.id)) {
                    return;
                }
                addedRuleSet.add(rule.id);
                const sarifRuleBuilder = new node_sarif_builder_1.SarifRuleBuilder().initSimple({
                    ruleId: rule.id,
                    shortDescriptionText: rule.description,
                    helpUri: rule.link,
                });
                sarifRunBuilder.addRule(sarifRuleBuilder);
            });
        });
        arrAllMessages.forEach((result) => {
            result.messages.forEach((message) => {
                const sarifResultBuilder = new node_sarif_builder_1.SarifResultBuilder();
                const ruleId = message.rule.id;
                const sarifResultInit = {
                    level: message.type === "info"
                        ? 'note'
                        : message.type.toString(),
                    messageText: message.message,
                    ruleId: ruleId,
                    fileUri: path
                        .relative(process.cwd(), result.file)
                        .replace(/\\/g, '/'),
                    startLine: message.line,
                    startColumn: message.col,
                    endLine: message.line,
                    endColumn: message.col,
                };
                sarifResultBuilder.initSimple(sarifResultInit);
                sarifRunBuilder.addResult(sarifResultBuilder);
            });
        });
        sarifBuilder.addRun(sarifRunBuilder);
        const sarifContent = sarifBuilder.buildSarifJsonString({ indent: true });
        try {
            const sarifJson = JSON.parse(sarifContent);
            const rules = sarifJson.runs[0].tool.driver.rules;
            rules.forEach((rule) => {
                const ruleMarkdown = getRuleMarkdown(rule.id);
                if (ruleMarkdown) {
                    rule.help = {
                        text: rule.shortDescription.text,
                        markdown: ruleMarkdown,
                    };
                }
            });
            const updatedSarifContent = JSON.stringify(sarifJson, null, 2);
            (0, fs_1.writeFileSync)('htmlhint.sarif', updatedSarifContent);
            console.log(updatedSarifContent);
        }
        catch (error) {
            (0, fs_1.writeFileSync)('htmlhint.sarif', sarifContent);
            console.log(sarifContent);
        }
    });
};
module.exports = sarifFormatter;
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoic2FyaWYuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi8uLi9zcmMvY2xpL2Zvcm1hdHRlcnMvc2FyaWYudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6Ijs7QUFBQSwyQkFBNEQ7QUFFNUQsMkRBSzJCO0FBQzNCLDZCQUE0QjtBQUk1QixNQUFNLEdBQUcsR0FBRyxPQUFPLENBQUMsdUJBQXVCLENBQUMsQ0FBQTtBQU81QyxTQUFTLGVBQWUsQ0FBQyxNQUFjO0lBRXJDLE1BQU0sYUFBYSxHQUFHO1FBRXBCLElBQUksQ0FBQyxJQUFJLENBQ1AsT0FBTyxDQUFDLEdBQUcsRUFBRSxFQUNiLFNBQVMsRUFDVCxLQUFLLEVBQ0wsU0FBUyxFQUNULE1BQU0sRUFDTixPQUFPLEVBQ1AsR0FBRyxNQUFNLE1BQU0sQ0FDaEI7UUFFRCxJQUFJLENBQUMsSUFBSSxDQUNQLElBQUksQ0FBQyxPQUFPLENBQUMsT0FBTyxDQUFDLE9BQU8sQ0FBQyx1QkFBdUIsQ0FBQyxDQUFDLEVBQ3RELFNBQVMsRUFDVCxLQUFLLEVBQ0wsU0FBUyxFQUNULE1BQU0sRUFDTixPQUFPLEVBQ1AsR0FBRyxNQUFNLE1BQU0sQ0FDaEI7UUFFRCxJQUFJLENBQUMsSUFBSSxDQUNQLE9BQU8sQ0FBQyxHQUFHLEVBQUUsRUFDYixLQUFLLEVBQ0wsU0FBUyxFQUNULE1BQU0sRUFDTixPQUFPLEVBQ1AsR0FBRyxNQUFNLE1BQU0sQ0FDaEI7S0FDRixDQUFBO0lBR0QsS0FBSyxNQUFNLFdBQVcsSUFBSSxhQUFhLEVBQUUsQ0FBQztRQUN4QyxJQUFJLENBQUM7WUFDSCxJQUFJLElBQUEsZUFBVSxFQUFDLFdBQVcsQ0FBQyxFQUFFLENBQUM7Z0JBQzVCLE1BQU0sT0FBTyxHQUFHLElBQUEsaUJBQVksRUFBQyxXQUFXLEVBQUUsTUFBTSxDQUFDLENBQUE7Z0JBR2pELE1BQU0sY0FBYyxHQUFHLE9BQU8sQ0FBQyxPQUFPLENBQUMsS0FBSyxFQUFFLENBQUMsQ0FBQyxHQUFHLENBQUMsQ0FBQTtn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