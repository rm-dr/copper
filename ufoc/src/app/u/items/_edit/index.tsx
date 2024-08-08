import styles from "./edit.module.scss";
import { Panel } from "@/app/components/panel";

import {
	XIconArrowRight,
	XIconAttrBinary,
	XIconEdit,
	XIconTrash,
	XIconUpload,
} from "@/app/components/icons";
import { ItemData, Selected } from "../page";
import { attrTypes } from "@/app/_util/attrs";
import { ActionIcon, Button, Text } from "@mantine/core";
import Image from "next/image";
import { ppBytes } from "@/app/_util/ppbytes";

export function EditPanel(params: { data: ItemData; select: Selected }) {
	const selected = params.data.data[params.select.selected[0]];
	console.log(selected);

	return (
		<>
			<Panel
				panel_id={styles.panel_edititem}
				icon={<XIconEdit />}
				title={"Edit items"}
			>
				<div className={styles.edit_container_rows}>
					{selected === undefined
						? null
						: Object.entries(selected.attrs)
								.sort()
								.map(([attr, val]: [string, any]) => {
									const d = attrTypes.find((x) => {
										return x.serialize_as === val.type;
									});
									if (d === undefined) {
										return null;
									}

									return (
										<div
											key={`${selected.idx}-${attr}`}
											className={styles.editrow}
										>
											<div className={styles.editrow_icon}>{d.icon}</div>
											<div className={styles.editrow_name}>{attr}</div>
											<div className={styles.editrow_value_old}>
												{d.editor.type === "inline"
													? d.editor.old_value({ attr: val })
													: null}
											</div>
											<div className={styles.editrow_value_new}>
												{d.editor.type === "inline"
													? d.editor.new_value({
															attr: val,
															onChange: console.log,
													  })
													: null}
											</div>
										</div>
									);
								})}
				</div>
				<div className={styles.edit_container_panel}>
					<div className={styles.paneltitle}>
						<div className={styles.paneltitle_icon}>
							<XIconAttrBinary />
						</div>
						<div className={styles.paneltitle_name}>cover_art</div>
					</div>
					<div className={styles.panelbody}>
						<div className={styles.panelimage}>
							<Image src="/cover.jpg" fill alt="Picture of the author" />
						</div>
					</div>
					<div className={styles.panelbottom}>
						<div>
							<Text>{ppBytes(100088)}</Text>
						</div>
						<div style={{ flexGrow: 1 }}>
							<Text ff="monospace">image/png</Text>
						</div>
						<div>
							<ActionIcon variant="filled" color="red">
								<XIconTrash style={{ width: "70%", height: "70%" }} />
							</ActionIcon>
						</div>
						<div>
							<ActionIcon variant="filled">
								<XIconUpload style={{ width: "70%", height: "70%" }} />
							</ActionIcon>
						</div>
					</div>
				</div>
			</Panel>
		</>
	);
}

function OldValue(params: { attr: any }) {
	<Button
		radius="0px"
		variant="outline"
		fullWidth
		rightSection={<XIconArrowRight />}
	>
		View in panel
	</Button>;
}

function NewValue(params: { attr: any }) {
	<Button radius="0px" variant="outline" fullWidth rightSection={<XIconEdit />}>
		Edit in panel
	</Button>;
}
