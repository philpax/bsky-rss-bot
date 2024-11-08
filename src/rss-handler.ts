import { datetime } from "ptera";
import { parseFeed } from "rss";
import Config from "./configuration.ts";
import { DatabaseHandler } from "./database-handler.ts";
import { Logger } from "./logger.ts";

export class RSSHandler {
    private filterDate: Date;
    private readonly feed: string;
    private readonly databaseHandler: DatabaseHandler;
    private readonly logger: Logger;

    public constructor(feed: string, idx: number, database: DatabaseHandler) {
        this.logger = new Logger(`RSSHandler${idx}`);
        this.logger.debug(`Initializing for feed ${feed}`);
        this.filterDate = datetime().subtract({
            hour: Config.getFeedBackdateHours(),
        }).toUTC().toJSDate();
        this.feed = feed;
        this.databaseHandler = database;
    }

    public async fetchValidUnposted() {
        const rssData = await parseFeed(
            await (await fetch(
                this.feed,
            )).text(),
        );
        const posts = rssData.entries.filter((post) => {
            return post.published &&
                post.published > this.filterDate;
        }).filter((post) => {
            const postUrl = post.links[0].href;
            if (!postUrl) {
                return false;
            }

            return !this.databaseHandler.hasPostedUrl(postUrl);
        });
        this.filterDate = datetime().toUTC().toJSDate();
        return posts;
    }
}
