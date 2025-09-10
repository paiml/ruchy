"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
    id: 'meta-charset-require',
    description: '<meta charset=""> must be present in <head> tag.',
    init(parser, reporter) {
        let headSeen = false;
        let metaCharsetSeen = false;
        let metaCharsetContent = '';
        let headEvent = null;
        const onTagStart = (event) => {
            const tagName = event.tagName.toLowerCase();
            if (tagName === 'head') {
                headSeen = true;
                headEvent = event;
            }
            else if (tagName === 'meta') {
                const mapAttrs = parser.getMapAttrs(event.attrs);
                if (mapAttrs['charset'] !== undefined) {
                    metaCharsetSeen = true;
                    metaCharsetContent = mapAttrs['charset'] || '';
                }
            }
        };
        parser.addListener('tagstart', onTagStart);
        parser.addListener('end', () => {
            if (headSeen && headEvent) {
                if (!metaCharsetSeen) {
                    reporter.error('<meta charset=""> must be present in <head> tag.', headEvent.line, headEvent.col, this, headEvent.raw);
                }
                else if (metaCharsetContent.trim() === '') {
                    reporter.error('<meta charset=""> value must not be empty.', headEvent.line, headEvent.col, this, headEvent.raw);
                }
            }
        });
    },
};
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibWV0YS1jaGFyc2V0LXJlcXVpcmUuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi8uLi9zcmMvY29yZS9ydWxlcy9tZXRhLWNoYXJzZXQtcmVxdWlyZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOztBQUdBLGtCQUFlO0lBQ2IsRUFBRSxFQUFFLHNCQUFzQjtJQUMxQixXQUFXLEVBQUUsa0RBQWtEO0lBQy9ELElBQUksQ0FBQyxNQUFNLEVBQUUsUUFBUTtRQUNuQixJQUFJLFFBQVEsR0FBRyxLQUFLLENBQUE7UUFDcEIsSUFBSSxlQUFlLEdBQUcsS0FBSyxDQUFBO1FBQzNCLElBQUksa0JBQWtCLEdBQUcsRUFBRSxDQUFBO1FBQzNCLElBQUksU0FBUyxHQUFpQixJQUFJLENBQUE7UUFFbEMsTUFBTSxVQUFVLEdBQWEsQ0FBQyxLQUFLLEVBQUUsRUFBRTtZQUNyQyxNQUFNLE9BQU8sR0FBRyxLQUFLLENBQUMsT0FBTyxDQUFDLFdBQVcsRUFBRSxDQUFBO1lBQzNDLElBQUksT0FBTyxLQUFLLE1BQU0sRUFBRSxDQUFDO2dCQUN2QixRQUFRLEdBQUcsSUFBSSxDQUFBO2dCQUNmLFNBQVMsR0FBRyxLQUFLLENBQUE7WUFDbkIsQ0FBQztpQkFBTSxJQUFJLE9BQU8sS0FBSyxNQUFNLEVBQUUsQ0FBQztnQkFDOUIsTUFBTSxRQUFRLEdBQUcsTUFBTSxDQUFDLFdBQVcsQ0FBQyxLQUFLLENBQUMsS0FBSyxDQUFDLENBQUE7Z0JBQ2hELElBQUksUUFBUSxDQUFDLFNBQVMsQ0FBQyxLQUFLLFNBQVMsRUFBRSxDQUFDO29CQUN0QyxlQUFlLEdBQUcsSUFBSSxDQUFBO29CQUN0QixrQkFBa0IsR0FBRyxRQUFRLENBQUMsU0FBUyxDQUFDLElBQUksRUFBRSxDQUFBO2dCQUNoRCxDQUFDO1lBQ0gsQ0FBQztRQUNILENBQUMsQ0FBQTtRQUVELE1BQU0sQ0FBQyxXQUFXLENBQUMsVUFBVSxFQUFFLFVBQVUsQ0FBQyxDQUFBO1FBQzFDLE1BQU0sQ0FBQyxXQUFXLENBQUMsS0FBSyxFQUFFLEdBQUcsRUFBRTtZQUM3QixJQUFJLFFBQVEsSUFBSSxTQUFTLEVBQUUsQ0FBQztnQkFDMUIsSUFBSSxDQUFDLGVBQWUsRUFBRSxDQUFDO29CQUNyQixRQUFRLENBQUMsS0FBSyxDQUNaLGtEQUFrRCxFQUNsRCxTQUFTLENBQUMsSUFBSSxFQUNkLFNBQVMsQ0FBQyxHQUFHLEVBQ2IsSUFBSSxFQUNKLFNBQVMsQ0FBQyxHQUFHLENBQ2QsQ0FBQTtnQkFDSCxDQUFDO3FCQUFNLElBQUksa0JBQWtCLENBQUMsSUFBSSxFQUFFLEtBQUssRUFBRSxFQUFFLENBQUM7b0JBQzVDLFFBQVEsQ0FBQyxLQUFLLENBQ1osNENBQTRDLEVBQzVDLFNBQVMsQ0FBQyxJQUFJLEVBQ2QsU0FBUyxDQUFDLEdBQUcsRUFDYixJQUFJLEVBQ0osU0FBUyxDQUFDLEdBQUcsQ0FDZCxDQUFBO2dCQUNILENBQUM7WUFDSCxDQUFDO1FBQ0gsQ0FBQyxDQUFDLENBQUE7SUFDSixDQUFDO0NBQ00sQ0FBQSJ9