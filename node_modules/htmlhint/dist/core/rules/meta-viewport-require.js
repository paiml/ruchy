"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
    id: 'meta-viewport-require',
    description: '<meta name="viewport"> with non-blank content must be present in <head> tag.',
    init(parser, reporter) {
        let headSeen = false;
        let metaViewportSeen = false;
        let metaViewportContent = '';
        let headEvent = null;
        const onTagStart = (event) => {
            const tagName = event.tagName.toLowerCase();
            if (tagName === 'head') {
                headSeen = true;
                headEvent = event;
            }
            else if (tagName === 'meta') {
                const mapAttrs = parser.getMapAttrs(event.attrs);
                if (mapAttrs['name'] && mapAttrs['name'].toLowerCase() === 'viewport') {
                    metaViewportSeen = true;
                    metaViewportContent = mapAttrs['content'] || '';
                }
            }
        };
        parser.addListener('tagstart', onTagStart);
        parser.addListener('end', () => {
            if (headSeen && headEvent) {
                if (!metaViewportSeen) {
                    reporter.error('<meta name="viewport"> must be present in <head> tag.', headEvent.line, headEvent.col, this, headEvent.raw);
                }
                else if (metaViewportContent.trim() === '') {
                    reporter.error('<meta name="viewport"> content attribute must not be empty.', headEvent.line, headEvent.col, this, headEvent.raw);
                }
            }
        });
    },
};
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibWV0YS12aWV3cG9ydC1yZXF1aXJlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL2NvcmUvcnVsZXMvbWV0YS12aWV3cG9ydC1yZXF1aXJlLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7O0FBR0Esa0JBQWU7SUFDYixFQUFFLEVBQUUsdUJBQXVCO0lBQzNCLFdBQVcsRUFDVCw4RUFBOEU7SUFDaEYsSUFBSSxDQUFDLE1BQU0sRUFBRSxRQUFRO1FBQ25CLElBQUksUUFBUSxHQUFHLEtBQUssQ0FBQTtRQUNwQixJQUFJLGdCQUFnQixHQUFHLEtBQUssQ0FBQTtRQUM1QixJQUFJLG1CQUFtQixHQUFHLEVBQUUsQ0FBQTtRQUM1QixJQUFJLFNBQVMsR0FBaUIsSUFBSSxDQUFBO1FBRWxDLE1BQU0sVUFBVSxHQUFhLENBQUMsS0FBSyxFQUFFLEVBQUU7WUFDckMsTUFBTSxPQUFPLEdBQUcsS0FBSyxDQUFDLE9BQU8sQ0FBQyxXQUFXLEVBQUUsQ0FBQTtZQUMzQyxJQUFJLE9BQU8sS0FBSyxNQUFNLEVBQUUsQ0FBQztnQkFDdkIsUUFBUSxHQUFHLElBQUksQ0FBQTtnQkFDZixTQUFTLEdBQUcsS0FBSyxDQUFBO1lBQ25CLENBQUM7aUJBQU0sSUFBSSxPQUFPLEtBQUssTUFBTSxFQUFFLENBQUM7Z0JBQzlCLE1BQU0sUUFBUSxHQUFHLE1BQU0sQ0FBQyxXQUFXLENBQUMsS0FBSyxDQUFDLEtBQUssQ0FBQyxDQUFBO2dCQUNoRCxJQUFJLFFBQVEsQ0FBQyxNQUFNLENBQUMsSUFBSSxRQUFRLENBQUMsTUFBTSxDQUFDLENBQUMsV0FBVyxFQUFFLEtBQUssVUFBVSxFQUFFLENBQUM7b0JBQ3RFLGdCQUFnQixHQUFHLElBQUksQ0FBQTtvQkFDdkIsbUJBQW1CLEdBQUcsUUFBUSxDQUFDLFNBQVMsQ0FBQyxJQUFJLEVBQUUsQ0FBQTtnQkFDakQsQ0FBQztZQUNILENBQUM7UUFDSCxDQUFDLENBQUE7UUFFRCxNQUFNLENBQUMsV0FBVyxDQUFDLFVBQVUsRUFBRSxVQUFVLENBQUMsQ0FBQTtRQUMxQyxNQUFNLENBQUMsV0FBVyxDQUFDLEtBQUssRUFBRSxHQUFHLEVBQUU7WUFDN0IsSUFBSSxRQUFRLElBQUksU0FBUyxFQUFFLENBQUM7Z0JBQzFCLElBQUksQ0FBQyxnQkFBZ0IsRUFBRSxDQUFDO29CQUN0QixRQUFRLENBQUMsS0FBSyxDQUNaLHVEQUF1RCxFQUN2RCxTQUFTLENBQUMsSUFBSSxFQUNkLFNBQVMsQ0FBQyxHQUFHLEVBQ2IsSUFBSSxFQUNKLFNBQVMsQ0FBQyxHQUFHLENBQ2QsQ0FBQTtnQkFDSCxDQUFDO3FCQUFNLElBQUksbUJBQW1CLENBQUMsSUFBSSxFQUFFLEtBQUssRUFBRSxFQUFFLENBQUM7b0JBQzdDLFFBQVEsQ0FBQyxLQUFLLENBQ1osNkRBQTZELEVBQzdELFNBQVMsQ0FBQyxJQUFJLEVBQ2QsU0FBUyxDQUFDLEdBQUcsRUFDYixJQUFJLEVBQ0osU0FBUyxDQUFDLEdBQUcsQ0FDZCxDQUFBO2dCQUNILENBQUM7WUFDSCxDQUFDO1FBQ0gsQ0FBQyxDQUFDLENBQUE7SUFDSixDQUFDO0NBQ00sQ0FBQSJ9