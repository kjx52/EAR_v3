-- SQL_ADD_USER Dump
-- 数据库添加用户脚本
-- version 1.0.2
--
-- 该脚本应在执行user_regist前调用。

DELIMITER //     

drop procedure if exists test;
CREATE PROCEDURE test (IN name1 CHAR(10), IN set_time1 INT, IN passwd1 VARCHAR(32), IN email1 VARCHAR(25))
BEGIN
    DECLARE tmp INT;
    DECLARE duplicate_exists BOOLEAN;

    REPEAT
        SET tmp = FLOOR(100 + (RAND() * 900));

        SELECT COUNT(*) > 0 INTO duplicate_exists
        FROM user_info
        WHERE user_id = tmp;

    UNTIL NOT duplicate_exists END REPEAT;

    INSERT INTO user_info (name, set_time, passwd, email, phone, user_id, borrowed_book, borrowed_num)
		VALUES (name1,
			set_time1,
            passwd1,
            email1,
            '0',
            tmp,
            '',
            '0'
		);
END;
//

DELIMITER ;
