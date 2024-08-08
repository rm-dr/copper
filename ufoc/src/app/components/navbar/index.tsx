import Link from "next/link";
import styles from "./navbar.module.scss";

import Banner from "../../../../public/banner.svg";

const Navbar = () => {
	return (
		<div className={styles.navbar}>
			<div className={styles.banner}>
				<Banner />
			</div>

			<div className={styles.usermenu}>
				<span className={styles.username}>User</span>
			</div>
		</div>
	);
};

export default Navbar;
