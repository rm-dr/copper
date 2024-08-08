import styles from "./edit.module.scss";
import { Panel } from "@/app/components/panel";

import { XIconArrowRight, XIconEdit } from "@/app/components/icons";
import { ItemData, Selected } from "../page";
import { attrTypes } from "@/app/_util/attrs";
import { Button } from "@mantine/core";

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
				<div className={styles.edit_container}>
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
			</Panel>
		</>
	);
}

