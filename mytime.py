# Script for summarizing time tracking information from daily notes.

from datetime import date
import os
import click
import dateutil
import logging
import pandas as pd
import pendulum
import re
from tabulate import tabulate

def extractTimeData(contents, prefix=''):
    td = []

    pattern = r'Time\.(\w+)\.(.+):\s*(\d+\.?\d?)'
    mobj = re.findall(pattern, contents)
    for (category, name, hours) in mobj:
        if prefix:
            td.append([prefix, category, name, float(hours)])
        else:
            td.append([category, name, float(hours)])
    return td

def getSummary(df, category):
    areadf = df.loc[df['Category'] == category].drop(columns=['Category'])
    areadf = areadf.groupby('Name')['Hours'].sum().reset_index()
    areadf = areadf.sort_values(by=['Hours'], ascending=False)
    total = areadf['Hours'].sum()
    areadf['%'] = areadf['Hours'] / total * 100
    return areadf, total

def getFilesInRange(fpath, begin, end):
    begindate = dateutil.parser.parse(begin).date()
    enddate = dateutil.parser.parse(end).date()

    files = []
    with os.scandir(fpath) as it:
        for entry in it:
            if entry.name.endswith(".md") and entry.is_file():
                filedate = dateutil.parser.parse(
                    os.path.basename(entry).split('.')[0]).date()
                if (begindate <= filedate) and (filedate <= enddate):
                    files.append(entry.path)
    return files

def gettimedata(files):
    timedata = []
    for entry in files:
        with open(entry, encoding='UTF-8') as f:
            name = os.path.splitext(
                os.path.basename(entry))[0]
            td = extractTimeData(f.read(), prefix=name)
            if len(td) > 0:
                timedata.extend(td)
    df = pd.DataFrame(
        timedata, columns=['File', 'Category', 'Name', 'Hours']).astype(
            {'Hours': 'float'})
    return df

def getNumFiles(df):
    return df['File'].nunique()

def printTable(table, tsv):
    tblFmt = 'github'
    if tsv: tblFmt = 'tsv'

    print(tabulate(
        table,
        headers='keys',
        showindex=False,
        floatfmt=('', '.1f', '.2f'),
        tablefmt=tblFmt))

def reportTimeSpent(path, category, begin, end, tsv=False):
    files = []
    try:
        files = getFilesInRange(path, begin, end)
        td = gettimedata(files)
        areas, total_hours = getSummary(td, category)
        days = getNumFiles(td)
        if total_hours:
            printTable(areas, tsv)
            print()
            print(f'Total hours: {total_hours}')
            print(f'Average hours/day: {total_hours/days}')
    except ValueError as err:
        print(f'Error parsing date: {err}')
        return

##########################################################################

def get_dates_thisweek():
    today = pendulum.today()
    start = today.start_of('week').to_date_string()
    end = today.end_of('week').to_date_string()
    return start, end

def get_dates_lastweek():
    lastweek = pendulum.today().subtract(weeks=1)
    start = lastweek.start_of('week').to_date_string()
    end = lastweek.end_of('week').to_date_string()
    return start, end

def get_dates_thismonth():
    today = pendulum.today()
    start = today.start_of('month').to_date_string()
    end = today.end_of('month').to_date_string()
    return start, end

def get_dates_lastmonth():
    lastmonth = pendulum.today().subtract(months=1)
    start = lastmonth.start_of('month').to_date_string()
    end = lastmonth.end_of('month').to_date_string()
    return start, end

def get_dates_thisyear():
    today = pendulum.today()
    start = today.start_of('year').to_date_string()
    end = today.end_of('year').to_date_string()
    return start, end

def get_dates_lastyear():
    lastyear = pendulum.today().subtract(years=1)
    start = lastyear.start_of('year').to_date_string()
    end = lastyear.end_of('year').to_date_string()
    return start, end

def get_dates(start, end, thisweek, thismonth, thisyear,
              lastweek, lastmonth, lastyear):
    start = start.strftime("%Y-%m-%d")
    end = end.strftime("%Y-%m-%d")
    if thisweek:
        start, end = get_dates_thisweek()
    elif thismonth:
        start, end = get_dates_thismonth()
    elif thisyear:
        start, end = get_dates_thisyear()
    elif lastweek:
        start, end = get_dates_lastweek()
    elif lastmonth:
        start, end = get_dates_lastmonth()
    elif lastyear:
        start, end = get_dates_lastyear()
    return start, end

##########################################################################

@click.command()
@click.version_option(version="0.1.0")
@click.option('--log', default='warning',
              help='Logging level (info, debug)')
@click.option('--path', default='.',
              help='Path to the input files.',
              type=click.Path(exists=True, file_okay=False))
@click.option('--category', default='Area',
              help="Category of time entries to summarise",
              type=click.Choice(['Area', 'Focus', 'Proj', 'Prof'],
                                case_sensitive=False))
@click.option('--tsv', default=False, is_flag=True,
              help='Format the output as tab separated values')
@click.option('--from', 'from_',
              default=pendulum.today(),
              help='Start of time tracking period (default is today).',
              type=click.DateTime())
@click.option('--to', default=pendulum.today(),
              help='End of time tracking period (default is today).',
              type=click.DateTime())
@click.option('--thisweek', default=False, is_flag=True,
              help="This week's time summary. Overrides --from and --to values.")
@click.option('--thismonth', default=False, is_flag=True,
              help="This month's time summary. Overrides --from and --to values.")
@click.option('--thisyear', default=False, is_flag=True,
              help="This year's time summary. Overrides --from and --to values.")
@click.option('--lastweek', default=False, is_flag=True,
              help="Last week's time summary. Overrides --from and --to values.")
@click.option('--lastmonth', default=False, is_flag=True,
              help="Last month's time summary. Overrides --from and --to values.")
@click.option('--lastyear', default=False, is_flag=True,
              help="Last year's time summary. Overrides --from and --to values.")
def mytime(log, path,
           category,
           tsv,
           from_, to,
           thisweek, thismonth, thisyear,
           lastweek, lastmonth, lastyear):
    """Summarize time tracking data."""
    # Logging setup
    numeric_level = getattr(logging, log.upper(), None)
    if not isinstance(numeric_level, int):
        raise ValueError(f"Invalid log level: {log}")
    logging.basicConfig(format='%(message)s', level=numeric_level)

    # Tune pandas settings
    pd.set_option('display.precision', 2)

    start, end = get_dates(from_, to,
                           thisweek, thismonth, thisyear,
                           lastweek, lastmonth, lastyear)
    logging.info(f'{start} -> {end}')

    reportTimeSpent(path, category.title(), start, end, tsv)


##########################################################################

if __name__ == "__main__":
    mytime()
