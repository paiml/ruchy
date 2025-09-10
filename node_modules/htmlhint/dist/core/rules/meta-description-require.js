"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
    id: 'meta-description-require',
    description: '<meta name="description"> with non-blank content must be present in <head> tag.',
    init(parser, reporter) {
        let headSeen = false;
        let metaDescriptionSeen = false;
        let metaDescriptionContent = '';
        let headEvent = null;
        const onTagStart = (event) => {
            const tagName = event.tagName.toLowerCase();
            if (tagName === 'head') {
                headSeen = true;
                headEvent = event;
            }
            else if (tagName === 'meta') {
                const mapAttrs = parser.getMapAttrs(event.attrs);
                if (mapAttrs['name'] &&
                    mapAttrs['name'].toLowerCase() === 'description') {
                    metaDescriptionSeen = true;
                    metaDescriptionContent = mapAttrs['content'] || '';
                }
            }
        };
        parser.addListener('tagstart', onTagStart);
        parser.addListener('end', () => {
            if (headSeen && headEvent) {
                if (!metaDescriptionSeen) {
                    reporter.error('<meta name="description"> must be present in <head> tag.', headEvent.line, headEvent.col, this, headEvent.raw);
                }
                else if (metaDescriptionContent.trim() === '') {
                    reporter.error('<meta name="description"> content attribute must not be empty.', headEvent.line, headEvent.col, this, headEvent.raw);
                }
            }
        });
    },
};
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibWV0YS1kZXNjcmlwdGlvbi1yZXF1aXJlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL2NvcmUvcnVsZXMvbWV0YS1kZXNjcmlwdGlvbi1yZXF1aXJlLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7O0FBR0Esa0JBQWU7SUFDYixFQUFFLEVBQUUsMEJBQTBCO0lBQzlCLFdBQVcsRUFDVCxpRkFBaUY7SUFDbkYsSUFBSSxDQUFDLE1BQU0sRUFBRSxRQUFRO1FBQ25CLElBQUksUUFBUSxHQUFHLEtBQUssQ0FBQTtRQUNwQixJQUFJLG1CQUFtQixHQUFHLEtBQUssQ0FBQTtRQUMvQixJQUFJLHNCQUFzQixHQUFHLEVBQUUsQ0FBQTtRQUMvQixJQUFJLFNBQVMsR0FBaUIsSUFBSSxDQUFBO1FBRWxDLE1BQU0sVUFBVSxHQUFhLENBQUMsS0FBSyxFQUFFLEVBQUU7WUFDckMsTUFBTSxPQUFPLEdBQUcsS0FBSyxDQUFDLE9BQU8sQ0FBQyxXQUFXLEVBQUUsQ0FBQTtZQUMzQyxJQUFJLE9BQU8sS0FBSyxNQUFNLEVBQUUsQ0FBQztnQkFDdkIsUUFBUSxHQUFHLElBQUksQ0FBQTtnQkFDZixTQUFTLEdBQUcsS0FBSyxDQUFBO1lBQ25CLENBQUM7aUJBQU0sSUFBSSxPQUFPLEtBQUssTUFBTSxFQUFFLENBQUM7Z0JBQzlCLE1BQU0sUUFBUSxHQUFHLE1BQU0sQ0FBQyxXQUFXLENBQUMsS0FBSyxDQUFDLEtBQUssQ0FBQyxDQUFBO2dCQUNoRCxJQUNFLFFBQVEsQ0FBQyxNQUFNLENBQUM7b0JBQ2hCLFFBQVEsQ0FBQyxNQUFNLENBQUMsQ0FBQyxXQUFXLEVBQUUsS0FBSyxhQUFhLEVBQ2hELENBQUM7b0JBQ0QsbUJBQW1CLEdBQUcsSUFBSSxDQUFBO29CQUMxQixzQkFBc0IsR0FBRyxRQUFRLENBQUMsU0FBUyxDQUFDLElBQUksRUFBRSxDQUFBO2dCQUNwRCxDQUFDO1lBQ0gsQ0FBQztRQUNILENBQUMsQ0FBQTtRQUVELE1BQU0sQ0FBQyxXQUFXLENBQUMsVUFBVSxFQUFFLFVBQVUsQ0FBQyxDQUFBO1FBQzFDLE1BQU0sQ0FBQyxXQUFXLENBQUMsS0FBSyxFQUFFLEdBQUcsRUFBRTtZQUM3QixJQUFJLFFBQVEsSUFBSSxTQUFTLEVBQUUsQ0FBQztnQkFDMUIsSUFBSSxDQUFDLG1CQUFtQixFQUFFLENBQUM7b0JBQ3pCLFFBQVEsQ0FBQyxLQUFLLENBQ1osMERBQTBELEVBQzFELFNBQVMsQ0FBQyxJQUFJLEVBQ2QsU0FBUyxDQUFDLEdBQUcsRUFDYixJQUFJLEVBQ0osU0FBUyxDQUFDLEdBQUcsQ0FDZCxDQUFBO2dCQUNILENBQUM7cUJBQU0sSUFBSSxzQkFBc0IsQ0FBQyxJQUFJLEVBQUUsS0FBSyxFQUFFLEVBQUUsQ0FBQztvQkFDaEQsUUFBUSxDQUFDLEtBQUssQ0FDWixnRUFBZ0UsRUFDaEUsU0FBUyxDQUFDLElBQUksRUFDZCxTQUFTLENBQUMsR0FBRyxFQUNiLElBQUksRUFDSixTQUFTLENBQUMsR0FBRyxDQUNkLENBQUE7Z0JBQ0gsQ0FBQztZQUNILENBQUM7UUFDSCxDQUFDLENBQUMsQ0FBQTtJQUNKLENBQUM7Q0FDTSxDQUFBIn0=