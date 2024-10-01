import { Node, NodeProps } from "@xyflow/react";
import { BaseNode } from "./base";

type ExtractTagsNodeType = Node<Record<string, never>, "extracttags">;

export function ExtractTagsNode({ id }: NodeProps<ExtractTagsNodeType>) {
	return (
		<>
			<BaseNode
				id={id}
				title={"Extract tags"}
				inputs={[{ id: "data", type: "Blob", tooltip: "Audio data" }]}
				outputs={[
					{ id: "Album", type: "Text", tooltip: "Album" },
					{ id: "AlbumArtist", type: "Text", tooltip: "Album artist" },
					{ id: "Comment", type: "Text", tooltip: "Comment" },
					{ id: "ReleaseDate", type: "Text", tooltip: "Release date" },
					{ id: "DiskNumber", type: "Text", tooltip: "Disk number" },
					{ id: "DiskTotal", type: "Text", tooltip: "Total disks" },
					{ id: "Genre", type: "Text", tooltip: "Genre" },
					{ id: "ISRC", type: "Text", tooltip: "ISRC" },
					{ id: "Lyrics", type: "Text", tooltip: "Lyrics" },
					{ id: "TrackNumber", type: "Text", tooltip: "Track number" },
					{ id: "TrackTotal", type: "Text", tooltip: "Total tracks" },
					{ id: "Title", type: "Text", tooltip: "Track title" },
					{ id: "Artist", type: "Text", tooltip: "Artist" },
					{ id: "Year", type: "Text", tooltip: "Year" },
				]}
			></BaseNode>
		</>
	);
}
