"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
    id: 'h1-require',
    description: '<h1> must be present in <body> tag and not be empty.',
    init(parser, reporter) {
        let bodyDepth = 0;
        let hasH1InBody = false;
        let bodyTagEvent = null;
        let currentH1Event = null;
        let h1IsEmpty = false;
        const onTagStart = (event) => {
            const tagName = event.tagName.toLowerCase();
            if (tagName === 'body') {
                bodyDepth++;
                if (bodyDepth === 1) {
                    hasH1InBody = false;
                    bodyTagEvent = event;
                }
            }
            else if (tagName === 'h1' && bodyDepth > 0) {
                hasH1InBody = true;
                currentH1Event = event;
                h1IsEmpty = true;
            }
        };
        const onText = (event) => {
            if (currentH1Event && h1IsEmpty) {
                if (event.raw && !/^\s*$/.test(event.raw)) {
                    h1IsEmpty = false;
                }
            }
        };
        const onTagEnd = (event) => {
            const tagName = event.tagName.toLowerCase();
            if (tagName === 'h1' && currentH1Event) {
                if (h1IsEmpty) {
                    reporter.warn('<h1> tag must not be empty.', currentH1Event.line, currentH1Event.col, this, currentH1Event.raw);
                }
                currentH1Event = null;
            }
            else if (tagName === 'body') {
                if (bodyDepth === 1 && !hasH1InBody && bodyTagEvent) {
                    reporter.warn('<h1> must be present in <body> tag.', bodyTagEvent.line, bodyTagEvent.col, this, bodyTagEvent.raw);
                }
                bodyDepth--;
                if (bodyDepth < 0)
                    bodyDepth = 0;
            }
        };
        parser.addListener('tagstart', onTagStart);
        parser.addListener('tagend', onTagEnd);
        parser.addListener('text', onText);
        parser.addListener('end', () => {
            if (bodyDepth > 0 && !hasH1InBody && bodyTagEvent) {
                reporter.warn('<h1> must be present in <body> tag.', bodyTagEvent.line, bodyTagEvent.col, this, bodyTagEvent.raw);
            }
        });
    },
};
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaDEtcmVxdWlyZS5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy9jb3JlL3J1bGVzL2gxLXJlcXVpcmUudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6Ijs7QUFHQSxrQkFBZTtJQUNiLEVBQUUsRUFBRSxZQUFZO0lBQ2hCLFdBQVcsRUFBRSxzREFBc0Q7SUFDbkUsSUFBSSxDQUFDLE1BQU0sRUFBRSxRQUFRO1FBQ25CLElBQUksU0FBUyxHQUFHLENBQUMsQ0FBQTtRQUNqQixJQUFJLFdBQVcsR0FBRyxLQUFLLENBQUE7UUFDdkIsSUFBSSxZQUFZLEdBQWlCLElBQUksQ0FBQTtRQUNyQyxJQUFJLGNBQWMsR0FBaUIsSUFBSSxDQUFBO1FBQ3ZDLElBQUksU0FBUyxHQUFHLEtBQUssQ0FBQTtRQUVyQixNQUFNLFVBQVUsR0FBYSxDQUFDLEtBQVksRUFBRSxFQUFFO1lBQzVDLE1BQU0sT0FBTyxHQUFHLEtBQUssQ0FBQyxPQUFPLENBQUMsV0FBVyxFQUFFLENBQUE7WUFDM0MsSUFBSSxPQUFPLEtBQUssTUFBTSxFQUFFLENBQUM7Z0JBQ3ZCLFNBQVMsRUFBRSxDQUFBO2dCQUNYLElBQUksU0FBUyxLQUFLLENBQUMsRUFBRSxDQUFDO29CQUNwQixXQUFXLEdBQUcsS0FBSyxDQUFBO29CQUNuQixZQUFZLEdBQUcsS0FBSyxDQUFBO2dCQUN0QixDQUFDO1lBQ0gsQ0FBQztpQkFBTSxJQUFJLE9BQU8sS0FBSyxJQUFJLElBQUksU0FBUyxHQUFHLENBQUMsRUFBRSxDQUFDO2dCQUM3QyxXQUFXLEdBQUcsSUFBSSxDQUFBO2dCQUNsQixjQUFjLEdBQUcsS0FBSyxDQUFBO2dCQUN0QixTQUFTLEdBQUcsSUFBSSxDQUFBO1lBQ2xCLENBQUM7UUFDSCxDQUFDLENBQUE7UUFFRCxNQUFNLE1BQU0sR0FBYSxDQUFDLEtBQVksRUFBRSxFQUFFO1lBRXhDLElBQUksY0FBYyxJQUFJLFNBQVMsRUFBRSxDQUFDO2dCQUNoQyxJQUFJLEtBQUssQ0FBQyxHQUFHLElBQUksQ0FBQyxPQUFPLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxHQUFHLENBQUMsRUFBRSxDQUFDO29CQUMxQyxTQUFTLEdBQUcsS0FBSyxDQUFBO2dCQUNuQixDQUFDO1lBQ0gsQ0FBQztRQUNILENBQUMsQ0FBQTtRQUVELE1BQU0sUUFBUSxHQUFhLENBQUMsS0FBWSxFQUFFLEVBQUU7WUFDMUMsTUFBTSxPQUFPLEdBQUcsS0FBSyxDQUFDLE9BQU8sQ0FBQyxXQUFXLEVBQUUsQ0FBQTtZQUMzQyxJQUFJLE9BQU8sS0FBSyxJQUFJLElBQUksY0FBYyxFQUFFLENBQUM7Z0JBQ3ZDLElBQUksU0FBUyxFQUFFLENBQUM7b0JBQ2QsUUFBUSxDQUFDLElBQUksQ0FDWCw2QkFBNkIsRUFDN0IsY0FBYyxDQUFDLElBQUksRUFDbkIsY0FBYyxDQUFDLEdBQUcsRUFDbEIsSUFBSSxFQUNKLGNBQWMsQ0FBQyxHQUFHLENBQ25CLENBQUE7Z0JBQ0gsQ0FBQztnQkFDRCxjQUFjLEdBQUcsSUFBSSxDQUFBO1lBQ3ZCLENBQUM7aUJBQU0sSUFBSSxPQUFPLEtBQUssTUFBTSxFQUFFLENBQUM7Z0JBQzlCLElBQUksU0FBUyxLQUFLLENBQUMsSUFBSSxDQUFDLFdBQVcsSUFBSSxZQUFZLEVBQUUsQ0FBQztvQkFDcEQsUUFBUSxDQUFDLElBQUksQ0FDWCxxQ0FBcUMsRUFDckMsWUFBWSxDQUFDLElBQUksRUFDakIsWUFBWSxDQUFDLEdBQUcsRUFDaEIsSUFBSSxFQUNKLFlBQVksQ0FBQyxHQUFHLENBQ2pCLENBQUE7Z0JBQ0gsQ0FBQztnQkFDRCxTQUFTLEVBQUUsQ0FBQTtnQkFDWCxJQUFJLFNBQVMsR0FBRyxDQUFDO29CQUFFLFNBQVMsR0FBRyxDQUFDLENBQUE7WUFDbEMsQ0FBQztRQUNILENBQUMsQ0FBQTtRQUVELE1BQU0sQ0FBQyxXQUFXLENBQUMsVUFBVSxFQUFFLFVBQVUsQ0FBQyxDQUFBO1FBQzFDLE1BQU0sQ0FBQyxXQUFXLENBQUMsUUFBUSxFQUFFLFFBQVEsQ0FBQyxDQUFBO1FBQ3RDLE1BQU0sQ0FBQyxXQUFXLENBQUMsTUFBTSxFQUFFLE1BQU0sQ0FBQyxDQUFBO1FBQ2xDLE1BQU0sQ0FBQyxXQUFXLENBQUMsS0FBSyxFQUFFLEdBQUcsRUFBRTtZQUU3QixJQUFJLFNBQVMsR0FBRyxDQUFDLElBQUksQ0FBQyxXQUFXLElBQUksWUFBWSxFQUFFLENBQUM7Z0JBQ2xELFFBQVEsQ0FBQyxJQUFJLENBQ1gscUNBQXFDLEVBQ3JDLFlBQVksQ0FBQyxJQUFJLEVBQ2pCLFlBQVksQ0FBQyxHQUFHLEVBQ2hCLElBQUksRUFDSixZQUFZLENBQUMsR0FBRyxDQUNqQixDQUFBO1lBQ0gsQ0FBQztRQUNILENBQUMsQ0FBQyxDQUFBO0lBQ0osQ0FBQztDQUNNLENBQUEifQ==