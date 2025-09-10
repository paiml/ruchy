"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
    id: 'main-require',
    description: '<main> must be present in <body> tag.',
    init(parser, reporter) {
        let bodyDepth = 0;
        let hasMainInBody = false;
        let bodyTagEvent = null;
        const onTagStart = (event) => {
            const tagName = event.tagName.toLowerCase();
            if (tagName === 'body') {
                bodyDepth++;
                if (bodyDepth === 1) {
                    hasMainInBody = false;
                    bodyTagEvent = event;
                }
            }
            else if (tagName === 'main' && bodyDepth > 0) {
                hasMainInBody = true;
            }
        };
        const onTagEnd = (event) => {
            const tagName = event.tagName.toLowerCase();
            if (tagName === 'body') {
                if (bodyDepth === 1 && !hasMainInBody && bodyTagEvent) {
                    reporter.warn('<main> must be present in <body> tag.', bodyTagEvent.line, bodyTagEvent.col, this, bodyTagEvent.raw);
                }
                bodyDepth--;
                if (bodyDepth < 0)
                    bodyDepth = 0;
            }
        };
        parser.addListener('tagstart', onTagStart);
        parser.addListener('tagend', onTagEnd);
        parser.addListener('end', () => {
            if (bodyDepth > 0 && !hasMainInBody && bodyTagEvent) {
                reporter.warn('<main> must be present in <body> tag.', bodyTagEvent.line, bodyTagEvent.col, this, bodyTagEvent.raw);
            }
        });
    },
};
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibWFpbi1yZXF1aXJlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL2NvcmUvcnVsZXMvbWFpbi1yZXF1aXJlLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7O0FBR0Esa0JBQWU7SUFDYixFQUFFLEVBQUUsY0FBYztJQUNsQixXQUFXLEVBQUUsdUNBQXVDO0lBQ3BELElBQUksQ0FBQyxNQUFNLEVBQUUsUUFBUTtRQUNuQixJQUFJLFNBQVMsR0FBRyxDQUFDLENBQUE7UUFDakIsSUFBSSxhQUFhLEdBQUcsS0FBSyxDQUFBO1FBQ3pCLElBQUksWUFBWSxHQUFpQixJQUFJLENBQUE7UUFFckMsTUFBTSxVQUFVLEdBQWEsQ0FBQyxLQUFZLEVBQUUsRUFBRTtZQUM1QyxNQUFNLE9BQU8sR0FBRyxLQUFLLENBQUMsT0FBTyxDQUFDLFdBQVcsRUFBRSxDQUFBO1lBQzNDLElBQUksT0FBTyxLQUFLLE1BQU0sRUFBRSxDQUFDO2dCQUN2QixTQUFTLEVBQUUsQ0FBQTtnQkFDWCxJQUFJLFNBQVMsS0FBSyxDQUFDLEVBQUUsQ0FBQztvQkFDcEIsYUFBYSxHQUFHLEtBQUssQ0FBQTtvQkFDckIsWUFBWSxHQUFHLEtBQUssQ0FBQTtnQkFDdEIsQ0FBQztZQUNILENBQUM7aUJBQU0sSUFBSSxPQUFPLEtBQUssTUFBTSxJQUFJLFNBQVMsR0FBRyxDQUFDLEVBQUUsQ0FBQztnQkFDL0MsYUFBYSxHQUFHLElBQUksQ0FBQTtZQUN0QixDQUFDO1FBQ0gsQ0FBQyxDQUFBO1FBRUQsTUFBTSxRQUFRLEdBQWEsQ0FBQyxLQUFZLEVBQUUsRUFBRTtZQUMxQyxNQUFNLE9BQU8sR0FBRyxLQUFLLENBQUMsT0FBTyxDQUFDLFdBQVcsRUFBRSxDQUFBO1lBQzNDLElBQUksT0FBTyxLQUFLLE1BQU0sRUFBRSxDQUFDO2dCQUN2QixJQUFJLFNBQVMsS0FBSyxDQUFDLElBQUksQ0FBQyxhQUFhLElBQUksWUFBWSxFQUFFLENBQUM7b0JBQ3RELFFBQVEsQ0FBQyxJQUFJLENBQ1gsdUNBQXVDLEVBQ3ZDLFlBQVksQ0FBQyxJQUFJLEVBQ2pCLFlBQVksQ0FBQyxHQUFHLEVBQ2hCLElBQUksRUFDSixZQUFZLENBQUMsR0FBRyxDQUNqQixDQUFBO2dCQUNILENBQUM7Z0JBQ0QsU0FBUyxFQUFFLENBQUE7Z0JBQ1gsSUFBSSxTQUFTLEdBQUcsQ0FBQztvQkFBRSxTQUFTLEdBQUcsQ0FBQyxDQUFBO1lBQ2xDLENBQUM7UUFDSCxDQUFDLENBQUE7UUFFRCxNQUFNLENBQUMsV0FBVyxDQUFDLFVBQVUsRUFBRSxVQUFVLENBQUMsQ0FBQTtRQUMxQyxNQUFNLENBQUMsV0FBVyxDQUFDLFFBQVEsRUFBRSxRQUFRLENBQUMsQ0FBQTtRQUN0QyxNQUFNLENBQUMsV0FBVyxDQUFDLEtBQUssRUFBRSxHQUFHLEVBQUU7WUFFN0IsSUFBSSxTQUFTLEdBQUcsQ0FBQyxJQUFJLENBQUMsYUFBYSxJQUFJLFlBQVksRUFBRSxDQUFDO2dCQUNwRCxRQUFRLENBQUMsSUFBSSxDQUNYLHVDQUF1QyxFQUN2QyxZQUFZLENBQUMsSUFBSSxFQUNqQixZQUFZLENBQUMsR0FBRyxFQUNoQixJQUFJLEVBQ0osWUFBWSxDQUFDLEdBQUcsQ0FDakIsQ0FBQTtZQUNILENBQUM7UUFDSCxDQUFDLENBQUMsQ0FBQTtJQUNKLENBQUM7Q0FDTSxDQUFBIn0=